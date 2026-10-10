//! Deterministic fuzz guards for Core external surfaces: document
//! DTO JSON, UUID text and content hashes.
//!
//! No fuzzer process is needed: seeded hostile corpora prove that
//! hostile input produces typed errors, never panics, hangs or
//! unbounded allocations beyond explicit budgets.

use petunia_core::{ContentHash, Document, ObjectId, ParentRef, SceneNode, VectorPath};

/// Tiny deterministic generator: same seeds, same corpus, every run.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn byte(&mut self) -> u8 {
        self.next() as u8
    }

    fn bytes(&mut self, len: usize) -> Vec<u8> {
        (0..len).map(|_| self.byte()).collect()
    }
}

fn valid_document_json() -> String {
    let mut document = Document::new("fuzz");
    let page = document.scene.default_page();
    document.scene.insert_node(SceneNode::new_path(
        "box",
        VectorPath::rect(0.0, 0.0, 10.0, 10.0),
        ParentRef::Page(page),
    ));
    document.to_json().expect("serializes")
}

/// Truncated inputs never panic and never parse.
#[test]
fn truncations_never_panic_nor_parse() {
    let json = valid_document_json();
    for fraction in [0, 1, 2, 3, 5, 7] {
        let end = json.len() * fraction / 8;
        // Floor to a character boundary; splitting UTF-8 itself must
        // not panic either.
        let mut end = end.min(json.len());
        while !json.is_char_boundary(end) {
            end -= 1;
        }
        let _ = Document::from_json(&json[..0]);
        assert!(
            Document::from_json(&json[..end]).is_err(),
            "truncated prefix parsed at fraction {fraction}/8",
        );
    }
}

/// Mutated inputs never panic; they fail with typed errors.
#[test]
fn mutations_never_panic() {
    let json = valid_document_json();
    let bytes = json.as_bytes();
    for seed in 0..256u64 {
        let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9) ^ 0x1234_5678);
        let mut mutated = bytes.to_vec();
        let hits = 1 + rng.byte() as usize % 6;
        for _ in 0..hits {
            let at = rng.byte() as usize % mutated.len().max(1);
            mutated[at] = rng.byte();
        }
        let text = String::from_utf8_lossy(&mutated).into_owned();
        let _ = Document::from_json(&text);
    }
}

/// Arbitrary bytes never panic the DTO layer.
#[test]
fn random_bytes_never_panic() {
    for seed in 0..256u64 {
        let bytes = Rng(seed).bytes(1 + (seed as usize * 4) % 256);
        let text = String::from_utf8_lossy(&bytes).into_owned();
        let _ = Document::from_json(&text);
        let _: Result<petunia_core::DocumentDtoV1, _> = serde_json::from_slice(&bytes);
    }
}

/// Deeply nested input hits the recursion budget, not the stack.
#[test]
fn deep_nesting_is_rejected() {
    let open = "[".repeat(10_000);
    let close = "]".repeat(10_000);
    assert!(Document::from_json(&format!("{open}{close}")).is_err());
    let open = "{\"a\":".repeat(10_000);
    let close = "1".to_string() + &"}".repeat(10_000);
    assert!(Document::from_json(&format!("{open}{close}")).is_err());
}

/// JSON scalars that are not documents fail with typed errors.
#[test]
fn non_documents_are_rejected() {
    for hostile in [
        "",
        "null",
        "true",
        "123",
        "\"str\"",
        "[]",
        "{}",
        "NaN",
        "Infinity",
        "{",
        "{\"schema_version\":",
    ] {
        assert!(Document::from_json(hostile).is_err(), "{hostile:?}");
    }
}

/// Out-of-range floats become infinity and are rejected by domain
/// validation, never stored.
#[test]
fn overflowing_floats_are_rejected() {
    let mut json = valid_document_json();
    json = json.replacen("10.0", "1e999", 1);
    assert!(Document::from_json(&json).is_err());
}

/// Byte budgets gate untrusted input before parsing.
#[test]
fn byte_budgets_are_enforced() {
    let json = valid_document_json();
    assert!(Document::from_json_limited(&json, json.len() as u64).is_ok());
    assert!(Document::from_json_limited(&json, json.len() as u64 - 1).is_err());
    assert!(Document::from_json_limited(&json, 0).is_err());
}

/// UUID text stays canonical: lowercase hyphenated serializes, and
/// only that shape parses back.
#[test]
fn uuid_text_is_canonical_and_strict() {
    let id = ObjectId::new_v4();
    let json = serde_json::to_string(&id).expect("serializes");
    assert_eq!(json, format!("\"{id}\""));
    assert_eq!(json, json.to_lowercase(), "no uppercase hex");
    assert!(json.contains('-'), "hyphenated");
    let back: ObjectId = serde_json::from_str(&json).expect("parses");
    assert_eq!(back, id);
    // Foreign spellings never slip into identity maps silently.
    for hostile in [
        id.to_string().to_uppercase(),
        id.as_uuid().simple().to_string(),
        format!("urn:uuid:{id}"),
        format!("{{{id}}}"),
        String::new(),
        "not-a-uuid".to_string(),
        "00000000-0000-0000-0000-00000000000".to_string(),
        "00000000-0000-0000-0000-0000000000000".to_string(),
    ] {
        assert!(
            serde_json::from_str::<ObjectId>(&format!("\"{hostile}\"")).is_err(),
            "{hostile:?}",
        );
    }
    // The UUID parser itself never panics on hostile text.
    for seed in 0..128u64 {
        let bytes = Rng(seed).bytes((seed as usize) % 40);
        let text = String::from_utf8_lossy(&bytes).into_owned();
        let _ = uuid::Uuid::parse_str(&text);
    }
}

/// Content hashes accept exactly 64 lowercase-or-uppercase hex
/// characters; everything else is a typed error, never a panic.
#[test]
fn content_hash_rejects_hostile_digests() {
    let good = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    assert!(ContentHash::from_hex(good).is_ok());
    assert!(ContentHash::from_hex(&good.to_uppercase()).is_ok());
    for hostile in [
        String::new(),
        "xyz".to_string(),
        "a".repeat(63),
        "a".repeat(65),
        "z".repeat(64),
        "aa".repeat(31) + "!",
        "é".repeat(32),
    ] {
        assert!(ContentHash::from_hex(&hostile).is_err(), "{hostile:?}");
    }
    for seed in 0..128u64 {
        let bytes = Rng(seed).bytes((seed as usize) % 80);
        let text = String::from_utf8_lossy(&bytes).into_owned();
        let _ = ContentHash::from_hex(&text);
    }
}

/// Duplicate identity keys fail with a typed error naming the key.
/// The last one never wins silently.
#[test]
fn duplicate_identity_keys_are_rejected() {
    let mut document = Document::new("dup");
    let page = document.scene.default_page();
    for name in ["a", "b"] {
        document.scene.insert_node(SceneNode::new_path(
            name,
            VectorPath::rect(0.0, 0.0, 1.0, 1.0),
            ParentRef::Page(page),
        ));
    }
    let json = document.to_json().expect("serializes");
    // Copy the first node entry inside the scene nodes map.
    let marker = "\"nodes\": {";
    let at = json.find(marker).expect("nodes map") + marker.len();
    let mut depth = 0usize;
    let mut end = at;
    for (index, byte) in json.bytes().enumerate().skip(at) {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    end = index + 1;
                    break;
                }
            }
            _ => {}
        }
    }
    assert!(end > at, "first entry found");
    let first = &json[at..end];
    let duped = format!("{}{},{}{}", &json[..at], first, first, &json[end..]);
    let error = Document::from_json(&duped).expect_err("must reject duplicates");
    assert!(
        error.to_string().contains("duplicate identity key"),
        "{error}",
    );
    // The honest document still parses.
    assert!(Document::from_json(&json).is_ok());
}
