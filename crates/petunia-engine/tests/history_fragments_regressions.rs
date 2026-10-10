//! Contract regressions for atomic authorial history and native clipboard.
use petunia_core::*;
use petunia_engine::fragments::*;
use petunia_engine::transaction::RegistryOp;
use petunia_engine::*;

fn request(operations: Vec<DocumentOp>) -> TransactionRequest {
    TransactionRequest {
        command_id: CommandId::new_v4(),
        operations,
        merge_key: None,
    }
}
fn commit(
    document: &mut Document,
    history: &mut History,
    operations: Vec<DocumentOp>,
) -> DocumentRevision {
    let prepared = prepare_transaction(document, request(operations), history.current_revision())
        .expect("prepare");
    history
        .commit(document, prepared, HistoryDescription::EditObjects)
        .expect("commit")
}
fn node(document: &mut Document, name: &str) -> ObjectId {
    let page = document.scene.default_page();
    let node = SceneNode::new_path(
        name,
        VectorPath::rect(1., 2., 8., 8.),
        ParentRef::Page(page),
    );
    let id = node.id;
    document.scene.insert_root(page, node).expect("insert");
    id
}
fn history() -> History {
    History::new(32 << 20, 64 << 20)
}
fn fragment(document: &Document, roots: &[ObjectId]) -> DocumentFragment {
    collect_fragment(
        document,
        roots,
        FragmentMetadata {
            source_document: document.id,
            source_revision: 0,
        },
    )
    .expect("fragment")
}
fn paste(
    document: &mut Document,
    history: &mut History,
    source: &DocumentFragment,
) -> (DocumentFragment, IdRemapping) {
    let map = IdRemapping::fresh_for_fragment(source);
    let remapped = remap_fragment(source, &map);
    let operations = fragment_into_parent_ops(
        document,
        &remapped,
        ParentRef::Page(document.scene.default_page()),
    )
    .expect("paste ops");
    commit(document, history, operations);
    (remapped, map)
}
fn json(document: &Document) -> serde_json::Value {
    serde_json::to_value(document).expect("json")
}

#[test]
fn branched_history_never_reuses_saved_or_discarded_revision() {
    let mut document = Document::new("branches");
    let object = node(&mut document, "box");
    let mut history = history();
    let saved = commit(
        &mut document,
        &mut history,
        vec![DocumentOp::SetVisibility {
            object,
            visible: false,
        }],
    );
    history.save_checkpoint();
    let discarded = commit(
        &mut document,
        &mut history,
        vec![DocumentOp::SetTransform {
            object,
            transform: Transform2D::translation(9., 0.),
        }],
    );
    history.undo(&mut document).expect("undo");
    assert!(!history.is_dirty());
    history.undo(&mut document).expect("undo");
    let branch = commit(
        &mut document,
        &mut history,
        vec![DocumentOp::SetTransform {
            object,
            transform: Transform2D::translation(2., 0.),
        }],
    );
    assert!(branch > discarded && branch > saved);
    assert!(history.is_dirty());
    assert!(!history.can_redo());
    history.undo(&mut document).expect("undo branch");
    history.redo(&mut document).expect("redo branch");
    assert_eq!(history.current_revision(), branch);
    assert!(history.is_dirty());
}

#[test]
fn save_completion_acknowledges_snapshot_revision_without_cleaning_new_edits() {
    let mut document = Document::new("save race");
    let object = node(&mut document, "box");
    let mut history = history();
    let snapshot = commit(
        &mut document,
        &mut history,
        vec![DocumentOp::SetVisibility {
            object,
            visible: false,
        }],
    );
    let current = commit(
        &mut document,
        &mut history,
        vec![DocumentOp::SetVisibility {
            object,
            visible: true,
        }],
    );
    history.mark_saved(snapshot).expect("save completes");
    assert_eq!(history.current_revision(), current);
    assert!(history.is_dirty());
    history.undo(&mut document).expect("undo to saved snapshot");
    assert!(!history.is_dirty());
    assert!(history.mark_saved(DocumentRevision(current.0 + 1)).is_err());
}

#[test]
fn paste_into_transformed_group_preserves_detached_world_position() {
    let mut source = Document::new("source");
    let object = node(&mut source, "box");
    source.scene.get_node_mut(object).unwrap().transform = Transform2D::translation(20., 30.);
    let original = source.scene.world_transform(object).unwrap();
    let copied = fragment(&source, &[object]);
    let map = IdRemapping::fresh_for_fragment(&copied);
    let remapped = remap_fragment(&copied, &map);
    let mut destination = Document::new("destination");
    let page = destination.scene.default_page();
    let mut group = SceneNode::new_path("group", VectorPath::new(), ParentRef::Page(page));
    group.item = SceneItem::Group(vec![]);
    group.transform = Transform2D::translation(100., 200.);
    let parent = group.id;
    destination.scene.insert_root(page, group).unwrap();
    let operations =
        fragment_into_parent_ops(&destination, &remapped, ParentRef::Object(parent)).unwrap();
    commit(&mut destination, &mut history(), operations);
    assert_eq!(
        destination
            .scene
            .world_transform(remapped.roots[0])
            .unwrap(),
        original
    );
}

#[test]
fn paragraph_grid_dependency_is_remapped_to_destination_page_and_undone_with_paste() {
    let mut source = Document::new("grid source");
    let object = node(&mut source, "text");
    let grid = GridDefinition::new(
        GridScope::Page(source.scene.default_page()),
        Point::new(0., 0.),
        GridSpec::Baseline(BaselineGridSpec::new(12., 0., 1).unwrap()),
    );
    let old_grid = grid.id;
    source.grids.insert(grid);
    let mut paragraph = ParagraphStyle::new(TextAlignment::Left, 12., 0., 0.).unwrap();
    paragraph.baseline_grid = Some(old_grid);
    let style = StyleId::new_v4();
    source
        .styles
        .insert(style, StyleDefinition::Paragraph(paragraph));
    source.scene.get_node_mut(object).unwrap().item = SceneItem::Text(
        TextObject::new(
            "A".into(),
            vec![],
            vec![ParagraphRun {
                range: TextRange { start: 0, end: 1 },
                style: ParagraphStyleRef { style },
            }],
            TextContainer::Artistic,
            TextFlow { next: None },
        )
        .unwrap(),
    );
    let copied = fragment(&source, &[object]);
    assert_eq!(copied.grids.len(), 1);
    let mut destination = Document::new("grid destination");
    let mut history = history();
    let (_, map) = paste(&mut destination, &mut history, &copied);
    let grid = destination.grids.get(map.grids[&old_grid]).unwrap();
    assert_eq!(
        grid.scope,
        GridScope::Page(destination.scene.default_page())
    );
    destination.validate().unwrap();
    history.undo(&mut destination).unwrap();
    assert!(destination.grids.is_empty());
    assert!(destination.styles.is_empty());
    history.redo(&mut destination).unwrap();
    destination.validate().unwrap();
}

#[test]
fn page_deletion_restores_original_owner_and_z_order() {
    let mut document = Document::new("root z order");
    let roots: Vec<_> = ["a", "b", "c", "d"]
        .map(|name| node(&mut document, name))
        .to_vec();
    let original = json(&document);
    let mut history = history();
    commit(
        &mut document,
        &mut history,
        vec![DocumentOp::RemoveObjects {
            roots: vec![roots[3], roots[1]],
        }],
    );
    assert_eq!(document.scene.root_order(), &[roots[0], roots[2]]);
    history.undo(&mut document).expect("restore");
    assert_eq!(json(&document), original);
    history.redo(&mut document).expect("remove again");
    assert_eq!(document.scene.root_order(), &[roots[0], roots[2]]);
}

#[test]
fn late_commit_failure_preserves_document_revision_and_redo() {
    let mut document = Document::new("failure");
    let object = node(&mut document, "box");
    let mut history = history();
    commit(
        &mut document,
        &mut history,
        vec![DocumentOp::SetVisibility {
            object,
            visible: false,
        }],
    );
    history.undo(&mut document).expect("undo");
    let mut prepared = prepare_transaction(
        &document,
        request(vec![DocumentOp::SetVisibility {
            object,
            visible: false,
        }]),
        history.current_revision(),
    )
    .expect("prepare");
    prepared.forward.push(DocumentOp::SetTransform {
        object: ObjectId::new_v4(),
        transform: Transform2D::IDENTITY,
    });
    let original = json(&document);
    assert!(history
        .commit(&mut document, prepared, HistoryDescription::EditObjects)
        .is_err());
    assert_eq!(json(&document), original);
    assert_eq!(history.current_revision(), DocumentRevision::GENESIS);
    assert!(history.can_redo());
    history.redo(&mut document).expect("original redo retained");
    assert!(!document.scene.get_node(object).expect("node").visible);
}

#[test]
fn hard_budget_includes_old_geometry_and_ignores_forged_inverse() {
    let mut document = Document::new("budget");
    let object = node(&mut document, "large old geometry");
    let mut contour = Contour::new(false);
    for i in 0..10_000 {
        contour.push_node(PathNode::line(Point::new(f64::from(i), 0.), NodeKind::Cusp));
    }
    let path = VectorPath {
        contours: vec![contour],
        fill_rule: FillRule::NonZero,
    };
    let SceneItem::Path(item) = &mut document.scene.get_node_mut(object).expect("node").item else {
        panic!("path");
    };
    item.path = path;
    let original = json(&document);
    let mut history = History::new(1024, 1024);
    let mut prepared = prepare_transaction(
        &document,
        request(vec![DocumentOp::ReplacePath {
            object,
            path: VectorPath::rect(0., 0., 1., 1.),
        }]),
        history.current_revision(),
    )
    .expect("prepare");
    prepared.inverse.clear();
    assert!(matches!(
        history.commit(&mut document, prepared, HistoryDescription::EditObjects),
        Err(EngineError::BudgetExceeded(_))
    ));
    assert_eq!(json(&document), original);
    assert!(history.is_empty());
}

#[test]
fn clip_dependency_paste_and_undo_are_one_atomic_edit() {
    let mut source = Document::new("clip source");
    let target = node(&mut source, "target");
    let clip = node(&mut source, "source");
    source.scene.get_node_mut(target).expect("target").clip =
        Some(ClipBinding::new(target, clip, BindingSourceUse::BindingOnly).expect("clip"));
    let copied = fragment(&source, &[target]);
    assert_eq!(copied.nodes.len(), 2);
    let mut destination = Document::new("destination");
    let mut history = history();
    let (pasted, map) = paste(&mut destination, &mut history, &copied);
    let copied_target = destination
        .scene
        .get_node(map.map_object(target))
        .expect("pasted target");
    assert_eq!(
        copied_target.clip.expect("clip").source,
        map.map_object(clip)
    );
    assert!(destination.validate().is_ok());
    let after = json(&destination);
    history
        .undo(&mut destination)
        .expect("undo both source and target");
    assert!(destination.scene.is_empty());
    history.redo(&mut destination).expect("redo");
    assert_eq!(json(&destination), after);
    assert!(pasted
        .roots
        .iter()
        .all(|id| destination.scene.get_node(*id).is_some()));
}

#[test]
fn subtree_paste_preserves_child_order_parents_and_all_geometry_identities() {
    let mut source = Document::new("nested");
    let parent = node(&mut source, "group");
    source.scene.get_node_mut(parent).expect("group").item = SceneItem::Group(Vec::new());
    let mut originals = Vec::new();
    for name in ["a", "b", "c"] {
        let child = SceneNode::new_path(
            name,
            VectorPath::rect(0., 0., 2., 2.),
            ParentRef::Object(parent),
        );
        originals.push(child.clone());
        source
            .scene
            .insert_child(parent, child, None)
            .expect("child");
    }
    let copied = fragment(&source, &[parent]);
    let mut destination = Document::new("destination");
    let mut history = history();
    let (_, map) = paste(&mut destination, &mut history, &copied);
    assert_eq!(
        destination
            .scene
            .children_of(map.map_object(parent))
            .expect("children"),
        originals
            .iter()
            .map(|node| map.map_object(node.id))
            .collect::<Vec<_>>()
    );
    for original in originals {
        let pasted = destination
            .scene
            .get_node(map.map_object(original.id))
            .expect("child");
        assert_eq!(pasted.parent, ParentRef::Object(map.map_object(parent)));
        let before = original.item_path().expect("before");
        let after = pasted.item_path().expect("after");
        assert_ne!(before.contours[0].id, after.contours[0].id);
        assert_ne!(
            before.contours[0].nodes[0].id,
            after.contours[0].nodes[0].id
        );
        assert_ne!(
            original.item_appearance().expect("before").items[0].id,
            pasted.item_appearance().expect("after").items[0].id
        );
    }
    history.undo(&mut destination).expect("undo");
    assert!(destination.scene.is_empty());
}

fn resource(document: &mut Document, kind: ResourceKind) -> ResourceId {
    let id = ResourceId::new_v4();
    document.resources.insert(ResourceRecord {
        id,
        kind,
        source: ResourceSource::new_linked("https://example.test/asset").expect("uri"),
        content_hash: None,
        metadata: ResourceMetadata::default(),
    });
    id
}
#[test]
fn text_style_swatch_spot_and_resource_closure_remap_and_undo_together() {
    let mut source = Document::new("type and ink");
    let icc = resource(&mut source, ResourceKind::IccProfile);
    let font = resource(&mut source, ResourceKind::Font);
    let spot = SpotColorId::new_v4();
    source.spots.insert(SpotColor {
        id: spot,
        name: "Brand ink".into(),
        alternate: ProcessColor {
            value: ProcessColorValue::Rgb(Rgba {
                r: 1.,
                g: 0.,
                b: 0.,
                alpha: 1.,
            }),
            space: ColorSpaceRef::EmbeddedIcc(icc),
        },
    });
    let swatch = SwatchId::new_v4();
    source.swatches.insert(Swatch {
        id: swatch,
        name: "Brand".into(),
        value: SwatchValue::Color(ColorValue::Spot(
            SpotColorRef::new(spot, 0.5).expect("tint"),
        )),
    });
    let style = StyleId::new_v4();
    source.styles.insert(
        style,
        StyleDefinition::Character(CharacterStyle {
            font: FontRef {
                family: "Example font".into(),
                style_name: None,
                resource: Some(font),
                axes: Vec::new(),
            },
            size: 12.,
            color: ColorSource::Swatch(swatch),
            tracking: 0.,
            baseline_shift: 0.,
            language: "pt-BR".into(),
            slant: FontSlant::Normal,
            features: Vec::new(),
        }),
    );
    let object = node(&mut source, "label");
    source.scene.get_node_mut(object).expect("text").item = SceneItem::Text(
        TextObject::new(
            "Hi".into(),
            vec![TextRun {
                range: TextRange::new(0, 2, "Hi").expect("range"),
                style: CharacterStyleRef { style },
            }],
            Vec::new(),
            TextContainer::Artistic,
            TextFlow { next: None },
        )
        .expect("text"),
    );
    let copied = fragment(&source, &[object]);
    assert_eq!(
        (
            copied.styles.len(),
            copied.swatches.len(),
            copied.spots.len(),
            copied.resources.len()
        ),
        (1, 1, 1, 2)
    );
    let mut destination = Document::new("destination");
    let mut history = history();
    let (_, map) = paste(&mut destination, &mut history, &copied);
    assert_ne!(map.styles[&style], style);
    assert_ne!(map.swatches[&swatch], swatch);
    assert_ne!(map.spots[&spot], spot);
    assert_ne!(map.resources[&font], font);
    assert!(destination.validate().is_ok());
    let after = json(&destination);
    history
        .undo(&mut destination)
        .expect("undo nodes and registries");
    assert!(
        destination.scene.is_empty()
            && destination.styles.is_empty()
            && destination.swatches.is_empty()
            && destination.spots.is_empty()
            && destination.resources.is_empty()
    );
    history.redo(&mut destination).expect("redo all");
    assert_eq!(json(&destination), after);
}

#[test]
fn registry_failure_never_leaves_first_record_inserted() {
    let mut document = Document::new("atomic registry");
    let id = ResourceId::new_v4();
    let record = ResourceRecord {
        id,
        kind: ResourceKind::Image,
        source: ResourceSource::new_linked("https://example.test/image").expect("uri"),
        content_hash: None,
        metadata: ResourceMetadata::default(),
    };
    let prepared = PreparedTransaction {
        expected_revision: DocumentRevision::GENESIS,
        command_id: CommandId::new_v4(),
        merge_key: None,
        affected_objects: Vec::new(),
        inverse: Vec::new(),
        forward: vec![
            DocumentOp::Registry(RegistryOp::Resource {
                id,
                value: Some(Box::new(record)),
            }),
            DocumentOp::SetVisibility {
                object: ObjectId::new_v4(),
                visible: false,
            },
        ],
    };
    assert!(commit_transaction(&mut document, prepared).is_err());
    assert!(document.resources.is_empty());
}

#[test]
fn effect_parameter_transaction_restores_captured_value() {
    let mut document = Document::new("effects");
    let object = node(&mut document, "box");
    let effect = EffectInstance::new(
        true,
        0.3,
        BlendMode::Normal,
        None,
        GeometryEffect::Corners(CornerParams::new(2.).expect("radius")),
    )
    .expect("effect");
    let effect_id = effect.id;
    document
        .scene
        .get_node_mut(object)
        .expect("node")
        .geometry_effects
        .items
        .push(effect);
    let mut history = history();
    commit(
        &mut document,
        &mut history,
        vec![DocumentOp::SetEffectParameter {
            effect: effect_id,
            parameter: EffectParameter::Opacity(0.8),
        }],
    );
    assert_eq!(
        document
            .scene
            .get_node(object)
            .expect("node")
            .geometry_effects
            .items[0]
            .opacity,
        0.8
    );
    history.undo(&mut document).expect("undo effect");
    assert_eq!(
        document
            .scene
            .get_node(object)
            .expect("node")
            .geometry_effects
            .items[0]
            .opacity,
        0.3
    );
}
