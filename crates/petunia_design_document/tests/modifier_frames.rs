use petunia_design_document::{
    ContainerRole, Document, DocumentMutator, DocumentObject, ModifierItem, ModifierKind,
    ModifierSpace, OpacityStop, ShapeKind,
};
use petunia_design_foundation::{ObjectId, SurfaceId};
use petunia_design_geometry::{GPoint, GRect, OffsetCap, OffsetJoin};

const ID: ObjectId = ObjectId::new(2);
fn fixture() -> Document {
    let mut doc = Document::new();
    let mut m = DocumentMutator::new(&mut doc);
    m.add_surface(SurfaceId::new(1), "Page").unwrap();
    let mut object = DocumentObject::new(ID, "Art");
    object.bounds = Some([100.0, 200.0, 100.0, 100.0]);
    object.shape = Some(ShapeKind::Rectangle {
        corner_radii: [0.0; 4],
    });
    m.add_object(SurfaceId::new(1), object).unwrap();
    doc
}
fn contour() -> ModifierItem {
    ModifierItem::enabled(
        1,
        ModifierKind::ContourOffset {
            distance: 10.0,
            join: OffsetJoin::Miter,
            cap: OffsetCap::None,
        },
    )
}
fn crop() -> ModifierItem {
    ModifierItem::enabled(
        2,
        ModifierKind::CropRect {
            rect: [110.0, 210.0, 50.0, 30.0],
        },
    )
}
fn gradient() -> ModifierItem {
    ModifierItem::enabled(
        3,
        ModifierKind::TransparentGradient {
            start: [100.0, 200.0],
            end: [200.0, 300.0],
            stops: vec![OpacityStop::new(0.0, 1.0), OpacityStop::new(1.0, 0.0)],
        },
    )
}
fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-6, "{a} != {b}");
}
fn bounds(doc: &Document, expected: GRect) {
    let got = doc
        .evaluated_path_world(ID)
        .unwrap()
        .bounding_box()
        .unwrap();
    for (a, b) in [
        (got.x0, expected.x0),
        (got.y0, expected.y0),
        (got.x1, expected.x1),
        (got.y1, expected.y1),
    ] {
        close(a, b);
    }
}

#[test]
fn schema_one_and_two_migrate_parent_parameters_without_changing_source() {
    for version in [1, 2] {
        let mut wire = serde_json::to_value(fixture()).unwrap();
        wire["schema_version"] = version.into();
        // Explicit old wire fixture: schema 1/2 did not contain a space marker.
        wire["surfaces"][0]["objects"][0]["modifiers"] = serde_json::json!([
            {"id":2,"kind":{"type":"CropRect","rect":[110,210,50,30]},"enabled":true},
            {"id":3,"kind":{"type":"TransparentGradient","start":[100,200],"end":[200,300],"stops":[{"offset":0,"opacity":1},{"offset":1,"opacity":0}]},"enabled":true}
        ]);
        let doc = Document::from_json(&wire.to_string()).unwrap();
        assert_eq!(
            doc.schema_version(),
            petunia_design_foundation::NATIVE_SCHEMA_VERSION
        );
        bounds(&doc, GRect::new(110.0, 210.0, 160.0, 240.0));
        close(
            doc.opacity_at_world(ID, GPoint::new(150.0, 250.0)).unwrap(),
            0.5,
        );
        assert!(doc
            .find_object(ID)
            .unwrap()
            .modifiers
            .iter()
            .all(|m| matches!(
                m.space,
                ModifierSpace::Local {
                    reference_size: [100.0, 100.0]
                }
            )));
        assert_eq!(Document::from_json(&doc.to_json().unwrap()).unwrap(), doc);
    }
}

#[test]
fn crop_and_diagonal_opacity_follow_movement_and_nonuniform_resize() {
    let mut doc = fixture();
    let mut m = DocumentMutator::new(&mut doc);
    m.set_modifiers(ID, vec![crop(), gradient()]).unwrap();
    let before = m.document().clone();
    let source = m.document().find_object(ID).unwrap().clone();
    let changes = m
        .set_bounds(ID, Some([-20.0, 70.0, 200.0, 50.0]), 0.0)
        .unwrap();
    bounds(m.document(), GRect::new(0.0, 75.0, 100.0, 90.0));
    // The reference-space sample is (100,0): t=0.5. Projecting onto a
    // resized diagonal would incorrectly produce t=0.8.
    close(
        m.document()
            .opacity_at_world(ID, GPoint::new(180.0, 70.0))
            .unwrap(),
        0.5,
    );
    let moved = m.document().find_object(ID).unwrap();
    assert_eq!(moved.modifiers, source.modifiers);
    assert_eq!(moved.shape, source.shape);
    m.revert(&changes).unwrap();
    assert_eq!(m.document(), &before);
}

#[test]
fn contour_distance_scales_in_both_axes_without_rewriting_parameters() {
    let mut doc = fixture();
    let mut m = DocumentMutator::new(&mut doc);
    m.set_modifiers(ID, vec![contour()]).unwrap();
    m.set_bounds(ID, Some([100.0, 200.0, 200.0, 50.0]), 0.0)
        .unwrap();
    bounds(m.document(), GRect::new(80.0, 195.0, 320.0, 255.0));
    assert!(matches!(
        m.document().find_object(ID).unwrap().modifiers[0].kind,
        ModifierKind::ContourOffset { distance: 10.0, .. }
    ));
}

#[test]
fn world_parameter_input_and_opacity_respect_rotated_ancestors() {
    let mut doc = fixture();
    let mut m = DocumentMutator::new(&mut doc);
    m.group_objects(
        SurfaceId::new(1),
        ObjectId::new(9),
        vec![ID],
        ContainerRole::Group,
    )
    .unwrap();
    m.set_bounds(ObjectId::new(9), Some([50.0, -30.0, 300.0, 200.0]), 0.7)
        .unwrap();
    m.set_bounds(ID, Some([10.0, 20.0, 100.0, 100.0]), 0.3)
        .unwrap();
    let world = m.document().world_transform_checked(ID).unwrap();
    let quad = [[0.0, 0.0], [100.0, 0.0], [100.0, 100.0], [0.0, 100.0]].map(|[x, y]| {
        let p = world.apply(GPoint::new(x, y));
        [p.x, p.y]
    });
    let item = m
        .document()
        .modifier_from_world(
            ID,
            ModifierItem::enabled(1, ModifierKind::Perspective { quad }),
        )
        .unwrap();
    let a = world.apply(GPoint::ORIGIN);
    let b = world.apply(GPoint::new(100.0, 100.0));
    let opacity = m
        .document()
        .modifier_from_world(
            ID,
            ModifierItem::enabled(
                3,
                ModifierKind::TransparentGradient {
                    start: [a.x, a.y],
                    end: [b.x, b.y],
                    stops: vec![OpacityStop::new(0.0, 1.0), OpacityStop::new(1.0, 0.0)],
                },
            ),
        )
        .unwrap();
    m.set_modifiers(ID, vec![item, opacity]).unwrap();
    let sample = world.apply(GPoint::new(100.0, 0.0));
    close(m.document().opacity_at_world(ID, sample).unwrap(), 0.5);
    let before = m.document().clone();
    assert!(m.document().modifier_from_world(ID, crop()).is_err());
    assert_eq!(m.document(), &before);
}

#[test]
fn disabled_modifiers_follow_placement_when_reenabled() {
    let mut doc = fixture();
    let mut item = crop();
    item.enabled = false;
    let mut m = DocumentMutator::new(&mut doc);
    m.set_modifiers(ID, vec![item]).unwrap();
    m.set_bounds(ID, Some([-20.0, 70.0, 200.0, 50.0]), 0.0)
        .unwrap();
    let mut chain = m.document().find_object(ID).unwrap().modifiers.clone();
    chain[0].enabled = true;
    m.set_modifiers(ID, chain).unwrap();
    bounds(m.document(), GRect::new(0.0, 75.0, 100.0, 90.0));
}

#[test]
fn invalid_replacement_and_unmarked_current_schema_fail_without_publication() {
    let mut doc = fixture();
    let before = doc.clone();
    let mut bad = crop();
    let local = crop()
        .into_local(Some([100.0, 200.0, 100.0, 100.0]))
        .unwrap();
    assert!(doc.modifier_from_world(ID, local).is_err());
    let negative = ModifierItem::enabled(
        1,
        ModifierKind::CropRect {
            rect: [150.0, 220.0, -20.0, 10.0],
        },
    );
    assert!(doc.modifier_from_world(ID, negative).is_err());
    bad.space = ModifierSpace::Local {
        reference_size: [0.0, 100.0],
    };
    assert!(DocumentMutator::new(&mut doc)
        .set_modifiers(ID, vec![bad])
        .is_err());
    assert_eq!(doc, before);
    assert!(DocumentMutator::new(&mut doc)
        .set_modifiers(ID, vec![crop(), crop()])
        .is_err());
    assert_eq!(doc, before);
    let mut wire = serde_json::to_value(doc).unwrap();
    wire["surfaces"][0]["objects"][0]["modifiers"] =
        serde_json::json!([{"id":2,"kind":{"type":"CropRect","rect":[110,210,50,30]}}]);
    assert!(Document::from_json(&wire.to_string()).is_err());
}

#[test]
fn empty_crop_hides_geometry_preserves_source_and_bakes_as_empty() {
    let mut doc = fixture();
    let mut m = DocumentMutator::new(&mut doc);
    let source = m.document().find_object(ID).unwrap().shape.clone();
    let changes = m.set_crop_rect(ID, [900.0, 900.0, 10.0, 10.0]).unwrap();
    assert!(m.document().evaluated_path_world(ID).unwrap().is_empty());
    assert_eq!(m.document().find_object(ID).unwrap().shape, source);
    let before = m.document().clone();
    let baked = m.bake_geometry(ID).unwrap();
    assert!(m.document().evaluated_path_world(ID).unwrap().is_empty());
    m.revert(&baked).unwrap();
    assert_eq!(m.document(), &before);
    m.revert(&changes).unwrap();
    assert!(!m.document().evaluated_path_world(ID).unwrap().is_empty());
}

#[test]
fn bake_preserves_rotated_world_outline_and_spatial_transparency() {
    let mut doc = fixture();
    let mut m = DocumentMutator::new(&mut doc);
    m.set_modifiers(ID, vec![contour(), gradient()]).unwrap();
    m.set_bounds(ID, Some([100.0, 200.0, 200.0, 50.0]), 0.7)
        .unwrap();
    let before = m.document().clone();
    let outline = before.evaluated_path_world(ID).unwrap().to_polygons(0.1);
    let world = before.world_transform_checked(ID).unwrap();
    let samples = [
        GPoint::new(0.0, 0.0),
        GPoint::new(80.0, 30.0),
        GPoint::new(200.0, 50.0),
    ]
    .map(|p| world.apply(p));
    let opacity = samples.map(|p| before.opacity_at_world(ID, p).unwrap());
    let baked = m.bake_geometry(ID).unwrap();
    let after = m
        .document()
        .evaluated_path_world(ID)
        .unwrap()
        .to_polygons(0.1);
    assert_eq!(outline.len(), after.len());
    for (a, b) in outline.iter().flatten().zip(after.iter().flatten()) {
        close(a.x, b.x);
        close(a.y, b.y);
    }
    for (p, expected) in samples.into_iter().zip(opacity) {
        close(m.document().opacity_at_world(ID, p).unwrap(), expected);
    }
    assert_eq!(m.document().find_object(ID).unwrap().modifiers.len(), 1);
    m.revert(&baked).unwrap();
    assert_eq!(m.document(), &before);
}

#[test]
fn contour_bake_does_not_apply_following_crop_twice_or_consume_it() {
    let mut doc = fixture();
    let mut m = DocumentMutator::new(&mut doc);
    m.set_modifiers(ID, vec![contour(), crop(), gradient()])
        .unwrap();
    let before = m.document().clone();
    let changes = m.bake_contour(ID).unwrap();
    bounds(m.document(), GRect::new(110.0, 210.0, 160.0, 240.0));
    assert_eq!(m.document().find_object(ID).unwrap().modifiers.len(), 2);
    close(
        m.document()
            .opacity_at_world(ID, GPoint::new(130.0, 230.0))
            .unwrap(),
        0.7,
    );
    m.revert(&changes).unwrap();
    assert_eq!(m.document(), &before);
}

#[test]
fn selective_bake_after_warp_fails_and_preserves_editable_chain() {
    let mut doc = fixture();
    let mut m = DocumentMutator::new(&mut doc);
    let warp = ModifierItem::enabled(
        4,
        ModifierKind::Perspective {
            quad: [
                [100.0, 200.0],
                [200.0, 210.0],
                [180.0, 290.0],
                [100.0, 300.0],
            ],
        },
    );
    m.set_modifiers(ID, vec![warp, contour()]).unwrap();
    let before = m.document().clone();
    assert!(m.bake_contour(ID).is_err());
    assert_eq!(m.document(), &before);
}

#[test]
fn opacity_sampling_preserves_unsorted_duplicate_stop_order() {
    let mut doc = fixture();
    let mut m = DocumentMutator::new(&mut doc);
    m.set_modifiers(
        ID,
        vec![ModifierItem::enabled(
            3,
            ModifierKind::TransparentGradient {
                start: [100.0, 200.0],
                end: [200.0, 200.0],
                stops: vec![
                    OpacityStop::new(0.75, 0.2),
                    OpacityStop::new(0.25, 0.9),
                    OpacityStop::new(0.25, 0.1),
                ],
            },
        )],
    )
    .unwrap();
    let before = m.document().find_object(ID).unwrap().modifiers.clone();
    close(
        m.document()
            .opacity_at_world(ID, GPoint::new(125.0, 200.0))
            .unwrap(),
        0.9,
    );
    close(
        m.document()
            .opacity_at_world(ID, GPoint::new(150.0, 200.0))
            .unwrap(),
        0.15,
    );
    close(
        m.document()
            .opacity_at_world(ID, GPoint::new(180.0, 200.0))
            .unwrap(),
        0.2,
    );
    assert_eq!(m.document().find_object(ID).unwrap().modifiers, before);
}
