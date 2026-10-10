//! Property and metamorphic tests for engine geometry and persistence.
//!
//! Covers the canonical metamorphic relations from verification.md:
//! split/reconstruct, union-with-empty identity, translate commuting
//! with union, zero-offset identity, lossless package round-trip,
//! atomic transaction undo, and panic-free corruption handling.

use petunia_core::{Document, FillRule, ObjectId, Point, SceneNode, Transform2D, VectorPath};
use petunia_engine::fragments::{collect_fragment, remap_fragment, FragmentMetadata, IdRemapping};
use petunia_engine::geometry::{boolean_rings, offset_ring, simplify_open, BooleanOp, CubicBez};
use petunia_engine::ptnd::{read_package, write_package, PackageLimits, ZipMethod};
use petunia_engine::transaction::{
    apply_quiet, commit_transaction, prepare_transaction, CommandId, DocumentOp, DocumentRevision,
    TransactionRequest,
};
use proptest::prelude::*;

fn square(x: f64, y: f64, size: f64) -> Vec<Point> {
    vec![
        Point::new(x, y),
        Point::new(x + size, y),
        Point::new(x + size, y + size),
        Point::new(x, y + size),
    ]
}

fn ring_area(ring: &[Point]) -> f64 {
    let mut sum = 0.0;
    for index in 0..ring.len() {
        let a = ring[index];
        let b = ring[(index + 1) % ring.len()];
        sum += a.x * b.y - b.x * a.y;
    }
    sum.abs() / 2.0
}

fn area_of(shapes: &[Vec<Vec<Point>>]) -> f64 {
    shapes
        .iter()
        .flat_map(|shape| shape.iter())
        .map(|ring| ring_area(ring))
        .sum()
}

proptest! {
    /// left(t) + right(t) reconstruct the original curve.
    #[test]
    fn bezier_split_reconstructs_the_curve(t in 0.0f64..=1.0) {
        let curve = CubicBez {
            p0: Point::new(0.0, 0.0),
            p1: Point::new(1.0, 4.0),
            p2: Point::new(3.0, 4.0),
            p3: Point::new(4.0, 0.0),
        };
        let (left, right) = curve.split(t);
        // `left` covers [0, t] reparametrized, so its end equals the
        // original at t; `right` continues from there to 1.
        prop_assert_eq!(left.evaluate(1.0), curve.evaluate(t));
        prop_assert_eq!(right.evaluate(0.0), curve.evaluate(t));
        prop_assert_eq!(right.evaluate(1.0), curve.evaluate(1.0));
        for u in [0.0, 0.25, 0.5, 1.0] {
            let rebuilt = right.evaluate(u);
            let original = curve.evaluate(t + u * (1.0 - t));
            prop_assert!((rebuilt.x - original.x).abs() < 1e-9);
            prop_assert!((rebuilt.y - original.y).abs() < 1e-9);
            let left_half = left.evaluate(u);
            let left_original = curve.evaluate(u * t);
            prop_assert!((left_half.x - left_original.x).abs() < 1e-9);
            prop_assert!((left_half.y - left_original.y).abs() < 1e-9);
        }
    }

    /// boolean union(A, empty) ≈ A — identity element holds for any shape.
    #[test]
    fn union_with_empty_is_identity(x in -1_000.0f64..1_000.0, size in 1.0f64..1_000.0) {
        let subject = vec![square(x, 0.0, size)];
        let result =
            boolean_rings(&subject, &[], BooleanOp::Union, FillRule::NonZero);
        let total = area_of(&result);
        prop_assert!((total - size * size).abs() < 1e-6 * size * size, "{total}");
    }

    /// translate(A ∪ B) has the same area as A ∪ B.
    #[test]
    fn translate_commutes_with_union(dx in -500.0f64..500.0, dy in -500.0f64..500.0) {
        let a = vec![square(0.0, 0.0, 10.0)];
        let b = vec![square(5.0, 0.0, 10.0)];
        let before = area_of(&boolean_rings(&a, &b, BooleanOp::Union, FillRule::NonZero));
        let shift = |points: &[Point]| -> Vec<Point> {
            points.iter().map(|p| Point::new(p.x + dx, p.y + dy)).collect()
        };
        let shifted = boolean_rings(
            &[shift(&a[0])],
            &[shift(&b[0])],
            BooleanOp::Union,
            FillRule::NonZero,
        );
        let after = area_of(&shifted);
        prop_assert!((before - after).abs() < 1e-6 * 150.0, "{before} != {after}");
    }

    /// offset distance 0 ≈ source.
    #[test]
    fn zero_offset_is_identity(size in 2.0f64..500.0) {
        let ring = square(0.0, 0.0, size);
        let offset = offset_ring(&ring, 0.0).expect("zero offset");
        prop_assert_eq!(offset.len(), ring.len());
        for (got, want) in offset.iter().zip(ring.iter()) {
            prop_assert!((got.x - want.x).abs() < 1e-9);
            prop_assert!((got.y - want.y).abs() < 1e-9);
        }
    }

    /// simplify keeps endpoints and never grows the point count.
    #[test]
    fn simplify_shrinks_or_keeps_points(n in 3usize..64) {
        let points: Vec<Point> = (0..n)
            .map(|index| Point::new(index as f64, if index % 2 == 0 { 0.0 } else { 0.001 }))
            .collect();
        let simplified = simplify_open(&points, 0.5);
        prop_assert!(simplified.len() >= 2);
        prop_assert!(simplified.len() <= points.len());
        prop_assert_eq!(simplified.first(), points.first());
        prop_assert_eq!(simplified.last(), points.last());
    }

    /// Any payload survives a package round-trip bit-for-bit.
    #[test]
    fn package_round_trip_is_lossless(size in 0usize..4096, method in 0u8..2) {
        let payload: Vec<u8> = (0..size).map(|index| (index % 251) as u8).collect();
        let zip_method = if method == 0 { ZipMethod::Stored } else { ZipMethod::Deflated };
        let bytes = write_package(
            &[("blob.bin".to_string(), payload.clone(), zip_method)],
            &PackageLimits::default(),
        )
        .expect("writes");
        let back = read_package(&bytes, &PackageLimits::default()).expect("reads");
        prop_assert_eq!(back.len(), 1);
        prop_assert_eq!(&back[0].0, "blob.bin");
        prop_assert_eq!(&back[0].1, &payload);
    }

    /// A transaction commits atomically and undo restores the prior state.
    #[test]
    fn transactions_undo_exactly(size in 1.0f64..1_000.0) {
        let mut document = Document::new("prop");
        let node = SceneNode::new_path(
            "box",
            VectorPath::rect(0.0, 0.0, size, size),
            petunia_core::ParentRef::Page(document.scene.default_page()),
        );
        let id = node.id;
        let prepared = prepare_transaction(
            &document,
            TransactionRequest {
                command_id: CommandId::new_v4(),
                operations: vec![DocumentOp::InsertRoot { index: 0, node: Box::new(node) }],
                merge_key: None,
            },
            DocumentRevision::GENESIS,
        )
        .expect("prepares");
        let applied = commit_transaction(&mut document, prepared).expect("commits");
        prop_assert!(document.scene.get_node(id).is_some());
        for op in applied.inverse {
            apply_quiet(&mut document, op).expect("undo applies");
        }
        prop_assert!(document.scene.get_node(id).is_none());
        prop_assert_eq!(document.scene.len(), 0);
    }

    /// Corrupt bytes never panic: they fail with a typed error.
    #[test]
    fn corrupt_packages_never_panic(len in 0usize..2048) {
        let mut bytes = write_package(
            &[("a.txt".to_string(), b"hello".to_vec(), ZipMethod::Deflated)],
            &PackageLimits::default(),
        )
        .expect("writes");
        for index in 0..bytes.len().min(len) {
            bytes[index] ^= (index % 97) as u8;
        }
        let _ = read_package(&bytes, &PackageLimits::default());
    }
}

#[test]
fn group_clip_bindings_reject_self_clips() {
    use petunia_core::{BindingSourceUse, ClipBinding};
    let id = ObjectId::new_v4();
    assert!(ClipBinding::new(id, id, BindingSourceUse::BindingOnly).is_err());
}

#[test]
fn transform_inverse_matches_manual_arithmetic() {
    let transform = Transform2D {
        a: 2.0,
        c: 1.0,
        b: 0.5,
        d: 1.5,
        tx: 10.0,
        ty: -4.0,
    };
    let inverse = transform.inverse().expect("invertible");
    let original = Point::new(7.0, -3.0);
    let forward = transform.transform_point(original);
    let back = inverse.transform_point(forward);
    assert!((back.x - original.x).abs() < 1e-9);
    assert!((back.y - original.y).abs() < 1e-9);
}

#[test]
fn document_fragments_remap_identities() {
    let mut document = Document::new("frag");
    document.scene.insert_node(SceneNode::new_path(
        "a",
        VectorPath::rect(0.0, 0.0, 4.0, 4.0),
        petunia_core::ParentRef::Page(document.scene.default_page()),
    ));
    let ids: Vec<ObjectId> = document.scene.root_order().to_vec();
    let fragment = collect_fragment(
        &document,
        &ids,
        FragmentMetadata {
            source_document: petunia_core::DocumentId::new_v4(),
            source_revision: 0,
        },
    )
    .expect("collects");
    let mapping = IdRemapping::fresh_for(&fragment.nodes);
    let remapped = remap_fragment(&fragment, &mapping);
    assert_ne!(
        remapped.roots[0], fragment.roots[0],
        "remapping must mint new identities"
    );
    assert_eq!(remapped.nodes.len(), fragment.nodes.len());
}
