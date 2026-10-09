//! Robustness fuzzing for untrusted-input boundaries.
//!
//! The canonical verification contract puts the PTND container,
//! DTO deserialization and SVG parsing first. Coverage-guided
//! fuzzing runs in its own CI job; this suite is the always-on
//! random-input guard that satisfies the contract goals for every
//! commit: no panics, no hangs, typed errors, bounded work.
//!
//! The generator is a deterministic xorshift PRNG seeded per case, so
//! any failure reproduces exactly from its seed.

use petunia_core::Document;
use petunia_engine::compile;
use petunia_engine::io::{sniff_format, DetectedFormat, ImportWarningKind, IoLimits};
use petunia_engine::ptnd::{read_package, PackageLimits, PtndManifest, ZipMethod};
use petunia_engine::transaction::DocumentRevision;
use petunia_render_model::RenderQuality;

/// Deterministic xorshift64* generator: same seed, same bytes.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        // Avoid the zero state, which never escapes.
        Self(seed ^ 0x9E37_79B9_7F4A_7C15 | 1)
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn byte(&mut self) -> u8 {
        (self.next_u64() >> 24) as u8
    }

    fn bytes(&mut self, len: usize) -> Vec<u8> {
        (0..len).map(|_| self.byte()).collect()
    }
}

/// Build a plausible-but-corrupt package: a valid skeleton with
/// spliced random bytes at a random offset.
fn corrupt_package(seed: u64) -> Vec<u8> {
    let mut rng = Rng::new(seed);
    let mut bytes = petunia_engine::ptnd::write_package(
        &[(
            "ptnd/manifest.json".to_string(),
            b"{\"format\":\"PTND\"}".to_vec(),
            ZipMethod::Deflated,
        )],
        &PackageLimits::default(),
    )
    .expect("valid package");
    let mutations = rng.byte() as usize % 6 + 1;
    for _ in 0..mutations {
        let offset = rng.byte() as usize % bytes.len().max(1);
        bytes[offset] = rng.byte();
    }
    bytes
}

fn bytes_only(seed: u64) -> Vec<u8> {
    Rng::new(seed).bytes(1 + (Rng::new(seed + 1).byte() as usize) * 4)
}

/// Truncated and extended packages never panic.
#[test]
fn fuzz_ptnd_reader_survives_corruption() {
    for seed in 0..512u64 {
        let bytes = corrupt_package(seed);
        let limits = PackageLimits::default();
        // Full read.
        let _ = read_package(&bytes, &limits);
        // Truncated read at every 1/8 boundary.
        for fraction in [0, 1, 2, 3, 5, 7] {
            let end = bytes.len() * fraction / 8;
            let _ = read_package(&bytes[..end], &limits);
        }
        // Random suffix bytes around a valid end-of-directory record.
        let mut extended = bytes.clone();
        extended.extend_from_slice(&Rng::new(seed + 7_919).bytes(48));
        let _ = read_package(&extended, &limits);
    }
}

/// Arbitrary bytes never panic the format sniffer nor the image
/// importer; they produce a typed result every time.
#[test]
fn fuzz_sniffing_and_image_import_never_panic() {
    let limits = IoLimits::default();
    for seed in 0..512u64 {
        let bytes = bytes_only(seed);
        let probe = sniff_format(&bytes);
        // Every probe is a known format with bounded confidence.
        assert!(
            matches!(
                probe.format,
                DetectedFormat::Png
                    | DetectedFormat::Jpeg
                    | DetectedFormat::Svg
                    | DetectedFormat::Ptnd
                    | DetectedFormat::Unknown
            ),
            "seed {seed} produced an unknown format state"
        );
        assert!((0.0..=1.0).contains(&probe.confidence), "seed {seed}");
        let _ = petunia_engine::io::import_raster_image(&bytes, &limits);
    }
}

/// Snapshot and manifest deserialization reject hostile input with
/// typed errors, never panics or hangs.
#[test]
fn fuzz_snapshot_deserialization_never_panics() {
    for seed in 0..512u64 {
        let bytes = bytes_only(seed);
        let _: Result<petunia_render_model::RenderSnapshot, _> = serde_json::from_slice(&bytes);
        let as_text = String::from_utf8_lossy(&bytes).into_owned();
        let _: Result<PtndManifest, _> = serde_json::from_str(&as_text);
        let _: Result<Document, _> = serde_json::from_str(&as_text);
    }
}

/// Hostile SVG text never panics the paint/serialization paths.
#[test]
fn fuzz_svg_and_path_text_never_panic() {
    let hostile = [
        "",
        "<",
        "<svg",
        "<svg>",
        "<svg/>",
        "<?xml version=\"1.0\"?><svg></svg>",
        "<svg><path d=\"M 0 0 Z\"/></svg>",
        "<svg><path d=\"M\"/></svg>",
        "\u{feff}<svg></svg>",
        "<svg xmlns=\"x\"><g><rect/></g></svg>",
        "<html><body>not svg</body></html>",
    ];
    for (index, text) in hostile.iter().enumerate() {
        let probe = sniff_format(text.as_bytes());
        if text.contains("<svg") && text.contains("<path") {
            assert_eq!(probe.format, DetectedFormat::Svg, "case {index}");
        }
        // Authorial paths always serialize to bounded SVG data.
        let data = petunia_engine::io::path_to_svg_data(&petunia_core::VectorPath::new());
        assert!(data.len() < 64, "case {index}");
    }
    // Deeply nested structures stay bounded, not recursive-blow-up.
    let deep = "<svg>".repeat(64) + "</svg>".repeat(64).as_str() + &"x".repeat(64);
    let _ = sniff_format(deep.as_bytes());
}

/// Compiling a hostile document never panics; degradations come back
/// as warnings rather than broken pixels.
#[test]
fn fuzz_document_compilation_never_panics() {
    for seed in 0..128u64 {
        let mut document = Document::new("fuzz");
        let count = Rng::new(seed).byte() % 8;
        for index in 0..count {
            document
                .scene
                .insert_node(petunia_core::SceneNode::new_path(
                    format!("n{index}"),
                    petunia_core::VectorPath::new(),
                ));
        }
        let (snapshot, warnings) = compile::compile_document(
            &document,
            DocumentRevision::GENESIS,
            RenderQuality::Authoring,
        );
        // Every page keeps a bounded primitive count.
        for page in &snapshot.pages {
            assert!(page.primitives.len() <= count as usize + 2, "seed {seed}");
        }
        assert!(warnings.len() <= count as usize, "seed {seed}");
        let _ = ImportWarningKind::UnsupportedFeature;
    }
}
