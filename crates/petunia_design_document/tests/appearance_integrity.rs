use petunia_design_document::{
    AdjustmentItem, AdjustmentKind, AppearanceStack, ChannelLevels, Document, DocumentMutator,
    DocumentObject, EffectItem, EffectKind,
};
use petunia_design_foundation::{ObjectId, SurfaceId};

fn document() -> Document {
    let mut doc = Document::new();
    let mut writer = DocumentMutator::new(&mut doc);
    writer.add_surface(SurfaceId::new(1), "Page").unwrap();
    writer
        .add_object(
            SurfaceId::new(1),
            DocumentObject::new(ObjectId::new(2), "Art"),
        )
        .unwrap();
    doc
}

fn adjustment(kind: AdjustmentKind) -> AppearanceStack {
    let mut stack = AppearanceStack::new();
    stack.add_adjustment(AdjustmentItem::new(1, kind));
    stack
}

#[test]
fn invalid_adjustments_fail_before_changing_appearance_or_legacy_fields() {
    let mut doc = document();
    let before = doc.clone();
    let kinds = [
        AdjustmentKind::Exposure {
            exposure: f64::NAN,
            offset: 0.0,
            gamma: 1.0,
        },
        AdjustmentKind::Hsl {
            hue_shift: 0.0,
            saturation: 2.0,
            lightness: 0.0,
        },
        AdjustmentKind::WhiteBalance {
            temperature: 0.0,
            tint: f64::INFINITY,
        },
        AdjustmentKind::Levels {
            master: ChannelLevels {
                input_white: 0.0,
                ..ChannelLevels::default()
            },
            red: None,
            green: None,
            blue: None,
        },
        AdjustmentKind::Curves {
            master_points: vec![[0.7, 0.0], [0.3, 1.0]],
            red_points: None,
            green_points: None,
            blue_points: None,
        },
    ];
    for kind in kinds {
        assert!(DocumentMutator::new(&mut doc)
            .set_appearance(ObjectId::new(2), Some(adjustment(kind)))
            .is_err());
        assert_eq!(doc, before);
    }
    let mut stack = adjustment(AdjustmentKind::default_levels());
    stack.adjustments[0].opacity = -0.1;
    assert!(DocumentMutator::new(&mut doc)
        .set_appearance(ObjectId::new(2), Some(stack))
        .is_err());
    assert_eq!(doc, before);
}

#[test]
fn invalid_effects_duplicate_ids_and_exhaustion_do_not_publish() {
    let mut doc = document();
    let before = doc.clone();
    for kind in [
        EffectKind::GaussianBlur { radius: -1.0 },
        EffectKind::DropShadow {
            offset: [f64::INFINITY, 0.0],
            blur: 0.0,
            color: "#000000".into(),
            opacity: 1.0,
        },
        EffectKind::InnerShadow {
            offset: [0.0, 0.0],
            blur: 0.0,
            color: "#000000".into(),
            opacity: f64::NAN,
        },
        EffectKind::Sharpen {
            radius: 1.0,
            amount: 6.0,
        },
        EffectKind::Noise {
            amount: 1.1,
            monochrome: false,
        },
    ] {
        let mut stack = AppearanceStack::new();
        stack.effects.push(EffectItem {
            id: 1,
            kind,
            visible: false,
        });
        assert!(DocumentMutator::new(&mut doc)
            .set_appearance(ObjectId::new(2), Some(stack))
            .is_err());
        assert_eq!(doc, before);
    }
    let mut duplicate = adjustment(AdjustmentKind::default_levels());
    duplicate.adjustments.push(duplicate.adjustments[0].clone());
    assert!(DocumentMutator::new(&mut doc)
        .set_appearance(ObjectId::new(2), Some(duplicate))
        .is_err());
    assert_eq!(doc, before);
    let mut stack = adjustment(AdjustmentKind::default_levels());
    stack.adjustments[0].id = u32::MAX;
    DocumentMutator::new(&mut doc)
        .set_appearance(ObjectId::new(2), Some(stack))
        .unwrap();
    let before = doc.clone();
    assert!(DocumentMutator::new(&mut doc)
        .add_adjustment(
            ObjectId::new(2),
            AdjustmentItem::new(u32::MAX, AdjustmentKind::default_levels())
        )
        .is_err());
    assert_eq!(doc, before);
}

#[test]
fn malformed_persisted_chains_are_rejected_and_inverted_curves_round_trip() {
    let mut doc = document();
    let stack = adjustment(AdjustmentKind::Curves {
        master_points: vec![[0.0, 1.0], [0.5, 0.5], [1.0, 0.0]],
        red_points: None,
        green_points: None,
        blue_points: None,
    });
    DocumentMutator::new(&mut doc)
        .set_appearance(ObjectId::new(2), Some(stack))
        .unwrap();
    let json = doc.to_json().unwrap();
    assert_eq!(Document::from_json(&json).unwrap(), doc);
    let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
    let points = &mut value["surfaces"][0]["objects"][0]["appearance"]["adjustments"][0]["kind"]
        ["master_points"];
    *points = serde_json::json!([[0.0, 0.0], [0.0, 1.0]]);
    assert!(Document::from_json(&value.to_string()).is_err());
}

#[test]
fn invalid_surface_geometry_layout_and_guides_preserve_prior_state() {
    use petunia_design_document::{Bleed, Guide, GuideOrientation, Margins};
    let mut doc = document();
    let before = doc.clone();
    let id = SurfaceId::new(1);
    let mut writer = DocumentMutator::new(&mut doc);
    for (origin, dimensions) in [
        ([f64::NAN, 0.0], [10.0, 10.0]),
        ([0.0, 0.0], [f64::INFINITY, 10.0]),
        ([0.0, 0.0], [0.0, 10.0]),
    ] {
        assert!(writer.set_surface_geometry(id, origin, dimensions).is_err());
        assert_eq!(writer.document(), &before);
    }
    assert!(writer.set_surface_bleed(id, Bleed::uniform(-1.0)).is_err());
    assert!(writer
        .set_surface_margins(id, Margins::uniform(f64::INFINITY))
        .is_err());
    assert!(writer
        .add_surface_guide(id, Guide::new(1, GuideOrientation::Horizontal, f64::NAN))
        .is_err());
    assert_eq!(writer.document(), &before);
    writer
        .set_surface_geometry(id, [-10.0, 20.0], [0.5, 0.25])
        .unwrap();
    assert_eq!(
        writer.document().surface(id).unwrap().dimensions,
        [0.5, 0.25]
    );
    writer
        .add_surface_guide(id, Guide::new(1, GuideOrientation::Horizontal, 3.0))
        .unwrap();
    let prior = writer.document().clone();
    assert!(writer
        .add_surface_guide(id, Guide::new(1, GuideOrientation::Vertical, 7.0))
        .is_err());
    assert_eq!(writer.document(), &prior);
}

#[test]
fn default_appearance_is_opaque_and_matches_the_explicit_constructor() {
    assert_eq!(AppearanceStack::default(), AppearanceStack::new());
    let mut doc = document();
    let appearance = AppearanceStack::default().with_fill("#ff0000");
    DocumentMutator::new(&mut doc)
        .set_appearance(ObjectId::new(2), Some(appearance))
        .unwrap();
    assert_eq!(
        doc.find_object(ObjectId::new(2))
            .unwrap()
            .effective_appearance()
            .opacity,
        1.
    );
}
