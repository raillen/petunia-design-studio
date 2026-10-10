//! Authorial layer locking and paint edits exercised through real history.

use petunia_core::*;
use petunia_engine::transaction::RegistryOp;
use petunia_engine::*;

fn request(operations: Vec<DocumentOp>) -> TransactionRequest {
    TransactionRequest {
        command_id: CommandId::new_v4(),
        operations,
        merge_key: None,
    }
}

fn fixture(shape: bool) -> (Document, ObjectId, History) {
    let mut document = Document::new("GUI paint and lock regression");
    let page = document.scene.default_page();
    let mut node = SceneNode::new_path(
        "Artwork",
        VectorPath::rect(10.0, 20.0, 120.0, 80.0),
        ParentRef::Page(page),
    );
    if shape {
        node.item = SceneItem::Shape(ShapeObject {
            shape: ParametricShape::Rectangle(RectangleSpec::new(
                Size2::new(120.0, 80.0).unwrap(),
                CornerRadii::uniform(Vec2::new(0.0, 0.0)).unwrap(),
            )),
            appearance: Appearance::solid_black(),
        });
    }
    let object = node.id;
    document.scene.insert_root(page, node).unwrap();
    (document, object, History::new(1 << 20, 2 << 20))
}

fn commit(document: &mut Document, history: &mut History, operations: Vec<DocumentOp>) {
    let prepared =
        prepare_transaction(document, request(operations), history.current_revision()).unwrap();
    history
        .commit(document, prepared, HistoryDescription::EditObjects)
        .unwrap();
}

fn paint() -> Appearance {
    let mut appearance = Appearance::solid_black();
    appearance.items[0].opacity = 0.65;
    appearance.items[0].kind = AppearanceKind::Fill(Paint::Solid(ColorSource::Value(
        ColorValue::Process(ProcessColor {
            value: ProcessColorValue::Rgb(Rgba {
                r: 0.8,
                g: 0.2,
                b: 0.1,
                alpha: 1.0,
            }),
            space: ColorSpaceRef::Builtin(BuiltinColorSpace::Srgb),
        }),
    )));
    appearance
}

fn snapshot(document: &Document) -> serde_json::Value {
    serde_json::to_value(document).unwrap()
}

#[test]
fn paint_and_lock_undo_redo_restore_paths_and_parametric_shapes_exactly() {
    for shape in [false, true] {
        let (mut document, object, mut history) = fixture(shape);
        let before = snapshot(&document);
        let appearance = paint();
        let prepared = prepare_transaction(
            &document,
            request(vec![
                DocumentOp::SetAppearance {
                    object,
                    appearance: appearance.clone(),
                },
                DocumentOp::SetLocked {
                    object,
                    locked: true,
                },
            ]),
            history.current_revision(),
        )
        .unwrap();
        assert_eq!(snapshot(&document), before, "prepare never mutates");
        assert_eq!(prepared.affected_objects, vec![object]);
        history
            .commit(&mut document, prepared, HistoryDescription::SetFill)
            .unwrap();
        let node = document.scene.get_node(object).unwrap();
        assert_eq!(node.item_appearance(), Some(&appearance));
        assert!(node.locked);
        assert!(history.is_dirty());
        let after = snapshot(&document);
        history.undo(&mut document).unwrap();
        assert_eq!(snapshot(&document), before);
        assert!(!history.is_dirty());
        history.redo(&mut document).unwrap();
        assert_eq!(snapshot(&document), after);
        assert!(history.is_dirty());
    }
}

#[test]
fn locked_layer_can_be_explicitly_unlocked_painted_and_relocked_atomically() {
    let (mut document, object, mut history) = fixture(false);
    commit(
        &mut document,
        &mut history,
        vec![DocumentOp::SetLocked {
            object,
            locked: true,
        }],
    );
    history.save_checkpoint();
    let before = snapshot(&document);
    let revision = history.current_revision();
    assert!(prepare_transaction(
        &document,
        request(vec![DocumentOp::SetAppearance {
            object,
            appearance: paint(),
        }]),
        revision,
    )
    .is_err());
    assert_eq!(snapshot(&document), before);
    assert_eq!(history.current_revision(), revision);
    commit(
        &mut document,
        &mut history,
        vec![
            DocumentOp::SetLocked {
                object,
                locked: false,
            },
            DocumentOp::SetAppearance {
                object,
                appearance: paint(),
            },
            DocumentOp::SetLocked {
                object,
                locked: true,
            },
        ],
    );
    let after = snapshot(&document);
    assert!(document.scene.get_node(object).unwrap().locked);
    history.undo(&mut document).unwrap();
    assert_eq!(snapshot(&document), before);
    assert!(!history.is_dirty());
    history.redo(&mut document).unwrap();
    assert_eq!(snapshot(&document), after);
}

#[test]
fn locked_group_protects_child_appearance_and_can_be_unlocked_in_same_request() {
    let (mut document, object, mut history) = fixture(false);
    let page = document.scene.default_page();
    let mut child = document.scene.remove_subtree(object).unwrap().remove(0);
    let mut group = SceneNode::new_path("Group", VectorPath::new(), ParentRef::Page(page));
    group.item = SceneItem::Group(vec![]);
    group.locked = true;
    let parent = group.id;
    document.scene.insert_root(page, group).unwrap();
    child.parent = ParentRef::Object(parent);
    document.scene.insert_child(parent, child, Some(0)).unwrap();
    assert!(prepare_transaction(
        &document,
        request(vec![DocumentOp::SetAppearance {
            object,
            appearance: paint(),
        }]),
        history.current_revision(),
    )
    .is_err());
    let appearance = paint();
    commit(
        &mut document,
        &mut history,
        vec![
            DocumentOp::SetLocked {
                object: parent,
                locked: false,
            },
            DocumentOp::SetAppearance {
                object,
                appearance: appearance.clone(),
            },
        ],
    );
    assert_eq!(
        document.scene.get_node(object).unwrap().item_appearance(),
        Some(&appearance)
    );
    history.undo(&mut document).unwrap();
    assert!(document.scene.get_node(parent).unwrap().locked);
}

#[test]
fn invalid_appearance_rolls_back_prior_lock_edits_for_path_and_shape() {
    for shape in [false, true] {
        let (mut document, object, history) = fixture(shape);
        document.scene.get_node_mut(object).unwrap().locked = true;
        let before = snapshot(&document);
        for opacity in [-0.1, 1.1, f32::NAN, f32::INFINITY] {
            let mut appearance = paint();
            appearance.items[0].opacity = opacity;
            assert!(prepare_transaction(
                &document,
                request(vec![
                    DocumentOp::SetLocked {
                        object,
                        locked: false
                    },
                    DocumentOp::SetAppearance { object, appearance },
                ]),
                history.current_revision(),
            )
            .is_err());
            assert_eq!(snapshot(&document), before);
            assert!(history.is_empty());
        }
    }
}

#[test]
fn invalid_color_and_missing_paint_resource_are_rejected_atomically() {
    let (document, object, history) = fixture(false);
    let before = snapshot(&document);
    let mut invalid_color = paint();
    let AppearanceKind::Fill(Paint::Solid(ColorSource::Value(ColorValue::Process(color)))) =
        &mut invalid_color.items[0].kind
    else {
        panic!("test fixture must use a process fill");
    };
    color.value = ProcessColorValue::Rgb(Rgba {
        r: f32::INFINITY,
        g: 0.0,
        b: 0.0,
        alpha: 1.0,
    });
    let mut missing_resource = paint();
    missing_resource.items[0].kind =
        AppearanceKind::Fill(Paint::Solid(ColorSource::Swatch(SwatchId::new_v4())));
    for appearance in [invalid_color, missing_resource] {
        assert!(prepare_transaction(
            &document,
            request(vec![
                DocumentOp::SetVisibility {
                    object,
                    visible: false
                },
                DocumentOp::SetAppearance { object, appearance },
            ]),
            history.current_revision(),
        )
        .is_err());
        assert_eq!(snapshot(&document), before);
    }
}

#[test]
fn paint_resource_can_be_inserted_in_same_transaction_and_history_restores_it() {
    let (mut document, object, mut history) = fixture(false);
    let before = snapshot(&document);
    let swatch = SwatchId::new_v4();
    let mut appearance = paint();
    let AppearanceKind::Fill(Paint::Solid(ColorSource::Value(value))) =
        appearance.items[0].kind.clone()
    else {
        panic!("test fixture must use a process fill");
    };
    appearance.items[0].kind = AppearanceKind::Fill(Paint::Solid(ColorSource::Swatch(swatch)));
    commit(
        &mut document,
        &mut history,
        vec![
            DocumentOp::Registry(RegistryOp::Swatch {
                id: swatch,
                value: Some(Box::new(Swatch {
                    id: swatch,
                    name: "Accent".into(),
                    value: SwatchValue::Color(value),
                })),
            }),
            DocumentOp::SetAppearance { object, appearance },
        ],
    );
    let after = snapshot(&document);
    assert!(document.swatches.get(swatch).is_some());
    assert!(prepare_transaction(
        &document,
        request(vec![DocumentOp::Registry(RegistryOp::Swatch {
            id: swatch,
            value: None
        })]),
        history.current_revision(),
    )
    .is_err());
    assert_eq!(snapshot(&document), after);
    history.undo(&mut document).unwrap();
    assert_eq!(snapshot(&document), before);
    assert!(document.swatches.get(swatch).is_none());
    history.redo(&mut document).unwrap();
    assert_eq!(snapshot(&document), after);
}

#[test]
fn unsupported_item_and_missing_object_leave_all_operations_uncommitted() {
    let (mut document, object, history) = fixture(false);
    document.scene.get_node_mut(object).unwrap().item = SceneItem::Group(vec![]);
    let before = snapshot(&document);
    for target in [object, ObjectId::new_v4()] {
        assert!(prepare_transaction(
            &document,
            request(vec![
                DocumentOp::SetLocked {
                    object,
                    locked: true
                },
                DocumentOp::SetAppearance {
                    object: target,
                    appearance: paint()
                },
            ]),
            history.current_revision(),
        )
        .is_err());
        assert_eq!(snapshot(&document), before);
    }
    assert!(prepare_transaction(
        &document,
        request(vec![DocumentOp::SetLocked {
            object: ObjectId::new_v4(),
            locked: false
        }]),
        history.current_revision(),
    )
    .is_err());
}

#[test]
fn commit_revalidates_new_lock_and_refuses_stale_prepared_paint() {
    let (mut document, object, mut history) = fixture(false);
    let prepared = prepare_transaction(
        &document,
        request(vec![DocumentOp::SetAppearance {
            object,
            appearance: paint(),
        }]),
        history.current_revision(),
    )
    .unwrap();
    document.scene.get_node_mut(object).unwrap().locked = true;
    let before = snapshot(&document);
    assert!(history
        .commit(&mut document, prepared, HistoryDescription::SetFill)
        .is_err());
    assert_eq!(snapshot(&document), before);
    assert!(history.is_empty());
    assert_eq!(history.current_revision(), DocumentRevision::GENESIS);
}

#[test]
fn invalid_stroke_and_geometry_cannot_commit_a_valid_appearance_prefix() {
    let (document, object, history) = fixture(false);
    let before = snapshot(&document);
    let mut appearance = paint();
    let AppearanceKind::Fill(Paint::Solid(source)) = appearance.items[0].kind.clone() else {
        panic!("test fixture must use a solid fill");
    };
    let mut stroke = StrokeStyle::new(2.0, StrokeCap::Round, StrokeJoin::Round, source).unwrap();
    stroke.width = -1.0;
    appearance.items[0].kind = AppearanceKind::Stroke(stroke);
    let requests = [
        vec![DocumentOp::SetAppearance { object, appearance }],
        vec![
            DocumentOp::SetAppearance {
                object,
                appearance: paint(),
            },
            DocumentOp::SetTransform {
                object,
                transform: Transform2D::translation(f64::INFINITY, 0.0),
            },
        ],
        vec![
            DocumentOp::SetAppearance {
                object,
                appearance: paint(),
            },
            DocumentOp::ReplacePath {
                object,
                path: VectorPath::rect(f64::INFINITY, 0.0, 10.0, 10.0),
            },
        ],
    ];
    for operations in requests {
        assert!(
            prepare_transaction(&document, request(operations), history.current_revision())
                .is_err()
        );
        assert_eq!(snapshot(&document), before);
    }
}
