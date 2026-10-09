//! Property tests for Core invariants.
//!
//! Properties follow the canonical verification contract: exact
//! equality for discrete values, tolerance-scoped equality for
//! geometry, and no global epsilon. Domain errors are asserted or
//! converted explicitly, so failures always name the invariant.

use petunia_core::{
    Angle, BlendMode, ContentHash, Document, DocumentId, ParametricShape, Point, ProcessColor,
    ProcessColorValue, Rect, SpotColorId, Tolerance, Transform2D, Unit, UnitValue, VectorPath,
};
use proptest::prelude::*;

fn finite_f64() -> impl Strategy<Value = f64> {
    -10_000.0f64..10_000.0f64
}

proptest! {
    /// inverse(T) × T × p ≈ p — canonical property from verification.md.
    #[test]
    fn transform_inverse_round_trips_any_point(
        a in 0.1f64..10.0,
        c in -5.0f64..5.0,
        b in -5.0f64..5.0,
        d in 0.1f64..10.0,
        tx in finite_f64(),
        ty in finite_f64(),
        px in finite_f64(),
        py in finite_f64(),
    ) {
        let transform = Transform2D { a, c, b, d, tx, ty };
        let inverse = transform.inverse().expect("non-degenerate matrix");
        let point = Point::new(px, py);
        let restored = inverse.transform_point(transform.transform_point(point));
        let scale = a.abs() + b.abs() + c.abs() + d.abs() + 1.0;
        prop_assert!(
            (restored.x - px).abs() < 1e-6 * scale && (restored.y - py).abs() < 1e-6 * scale,
            "{restored:?} != ({px}, {py})"
        );
    }

    /// decode(encode(x)) == canonical(x) for authorial units.
    #[test]
    fn unit_conversions_preserve_value(value in -1_000.0f64..1_000.0) {
        for unit in [Unit::Pt, Unit::Mm, Unit::Cm, Unit::Inch, Unit::Pica] {
            let start = UnitValue::new(value, unit).expect("finite value");
            let other = match unit {
                Unit::Pt => Unit::Mm,
                _ => Unit::Pt,
            };
            let there = start.to_unit(other).expect("physical to physical");
            let back = there.to_unit(unit).expect("physical to physical");
            prop_assert!(
                (back.value - value).abs() < 1e-9 * value.abs().max(1.0),
                "{unit:?} round trip drifted to {back:?}"
            );
        }
    }

    /// -0.0 canonicalizes at every public boundary.
    #[test]
    fn negative_zero_canonicalizes(seed in -1.0f64..1.0) {
        let value = if seed < 0.0 { -0.0 } else { seed };
        let parsed = UnitValue::new(value, Unit::Mm).expect("finite value");
        prop_assert!(parsed.value.is_sign_positive() || parsed.value > 0.0);
    }

    /// Document persistence keeps geometry exactly.
    #[test]
    fn document_serialization_never_loses_geometry(
        width in 1.0f64..5_000.0,
        height in 1.0f64..5_000.0,
    ) {
        let mut document = Document::new("prop");
        document.setup.width = width;
        document.setup.height = height;
        document
            .scene
            .insert_node(petunia_core::SceneNode::new_path(
                "box",
                VectorPath::rect(0.0, 0.0, width, height),
            ));
        let json = document.to_json().expect("serializes");
        let back = Document::from_json(&json).expect("deserializes");
        prop_assert_eq!(back.setup.width, width);
        prop_assert_eq!(back.setup.height, height);
        prop_assert_eq!(back.scene.len(), document.scene.len());
    }

    /// Parametric shapes persist without losing parameters.
    #[test]
    fn parametric_shapes_survive_serialization(
        sides in 3u32..64,
        radius in 0.0f64..5_000.0,
        rotation_deg in -720.0f64..720.0,
    ) {
        let shape = ParametricShape::Polygon(
            petunia_core::PolygonSpec::new(
                sides,
                radius,
                Angle::from_degrees(rotation_deg).expect("finite angle"),
            )
            .expect("valid polygon"),
        );
        let json = serde_json::to_string(&shape).expect("serializes");
        let back: ParametricShape = serde_json::from_str(&json).expect("deserializes");
        prop_assert_eq!(shape, back);
    }

    /// Alpha out of 0..1 is rejected; NaN/Inf never enters the domain.
    #[test]
    fn process_color_rejects_invalid_alpha(
        r in -5.0f32..5.0,
        alpha in -2.0f32..2.0,
    ) {
        let color = ProcessColor {
            value: ProcessColorValue::Rgb(petunia_core::Rgba {
                r,
                g: 0.0,
                b: 0.0,
                alpha,
            }),
            space: petunia_core::ColorSpaceRef::Builtin(petunia_core::BuiltinColorSpace::Srgb),
        };
        let expected = alpha.is_finite() && (0.0..=1.0).contains(&alpha) && r.is_finite();
        prop_assert_eq!(color.validate().is_ok(), expected);
    }

    /// Tolerance decides coincidence only inside its own band.
    #[test]
    fn tolerance_only_decides_coincidence(
        left in finite_f64(),
        right in finite_f64(),
        tolerance in 0.0f64..1_000.0,
    ) {
        let band = Tolerance::new(tolerance).expect("non-negative");
        let expected = (left - right).abs() <= tolerance;
        prop_assert_eq!(band.close_enough(left, right), expected);
    }

    /// Spot tint stays inside 0..1 for every generated input.
    #[test]
    fn spot_tint_bounds_hold(tint in -1.0f32..2.0) {
        let built = petunia_core::SpotColorRef::new(SpotColorId::new_v4(), tint);
        let expected = tint.is_finite() && (0.0..=1.0).contains(&tint);
        prop_assert_eq!(built.is_ok(), expected, "tint = {}", tint);
    }

    /// Content hashes parse only well-formed 64-char hex digests.
    #[test]
    fn content_hash_rejects_malformed_hex(len in 0usize..80) {
        let digest = "a".repeat(len);
        prop_assert_eq!(ContentHash::from_hex(&digest).is_ok(), len == 64);
    }

    /// Rects contain their centers and exclude one pixel outside.
    #[test]
    fn rect_contains_exactly_its_bounds(
        x in finite_f64(),
        y in finite_f64(),
        w in 1.0f64..1_000.0,
    ) {
        let rect = Rect::new(x, y, w, w);
        prop_assert!(rect.contains_point(Point::new(x + w / 2.0, y + w / 2.0)));
        prop_assert!(!rect.contains_point(Point::new(x + w + 1.0, y + w / 2.0)));
    }
}

/// Blend vocabulary stays stable across names in storage.
#[test]
fn blend_mode_serde_roundtrip() {
    for mode in [
        BlendMode::Normal,
        BlendMode::Multiply,
        BlendMode::Luminosity,
    ] {
        let json = serde_json::to_string(&mode).expect("serializes");
        let back: BlendMode = serde_json::from_str(&json).expect("deserializes");
        assert_eq!(mode, back);
    }
}

/// Typed IDs never collide across large samples.
#[test]
fn document_ids_never_collide() {
    let ids: Vec<DocumentId> = (0..1_000).map(|_| DocumentId::new_v4()).collect();
    let unique: std::collections::HashSet<_> = ids.iter().collect();
    assert_eq!(unique.len(), ids.len());
}
