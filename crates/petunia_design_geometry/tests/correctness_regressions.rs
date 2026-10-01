use petunia_design_geometry::{
    boolean_op, offset_path, simplify_rdp, BooleanInput, BooleanOp, GPath, GPoint, GRect,
    OffsetCap, OffsetJoin,
};

#[test]
fn boolean_empty_operands_obey_all_set_identities() {
    let shape = BooleanInput::single(vec![
        GPoint::new(0.0, 0.0),
        GPoint::new(20.0, 0.0),
        GPoint::new(0.0, 20.0),
    ]);
    for empty in [
        BooleanInput::new(vec![]),
        BooleanInput::single(vec![]),
        BooleanInput::single(vec![GPoint::ORIGIN]),
    ] {
        for operation in [BooleanOp::Union, BooleanOp::Xor] {
            assert_eq!(boolean_op(&shape, &empty, operation), shape.contours);
            assert_eq!(boolean_op(&empty, &shape, operation), shape.contours);
        }
        assert_eq!(
            boolean_op(&shape, &empty, BooleanOp::Difference),
            shape.contours
        );
        assert!(boolean_op(&empty, &shape, BooleanOp::Difference).is_empty());
        assert!(boolean_op(&shape, &empty, BooleanOp::Intersection).is_empty());
        assert!(boolean_op(&empty, &shape, BooleanOp::Intersection).is_empty());
    }
}

#[test]
fn simplify_preserves_excursions_beyond_a_degenerate_or_short_chord() {
    for end in [GPoint::ORIGIN, GPoint::new(10.0, 0.0)] {
        let points = vec![GPoint::ORIGIN, GPoint::new(20.0, 0.0), end];
        assert_eq!(simplify_rdp(&points, 0.1), points);
    }
}

#[test]
fn offset_preserves_disconnected_components_in_both_directions() {
    let mut path = GPath::rect(GRect::new(0.0, 0.0, 20.0, 20.0), 0.0, 0.0);
    path.verbs
        .extend(GPath::rect(GRect::new(100.0, 0.0, 120.0, 20.0), 0.0, 0.0).verbs);
    for distance in [2.0, -2.0] {
        let offset = offset_path(&path, distance, OffsetJoin::Round, OffsetCap::Round).unwrap();
        assert_eq!(offset.subpath_count(), 2, "distance {distance}");
        let bounds = offset.bounding_box().unwrap();
        assert!(bounds.x0 < 5.0 && bounds.x1 > 115.0);
    }
}
