//! Tests for UI panels (Layers, Properties, History).

use aubrieta_application::{Command, CommandRequest};
use aubrieta_foundation::IdGenerator;
use aubrieta_shell::bridge::AubrietaGuiBridge;
use aubrieta_shell::panels::*;

#[test]
fn layers_panel_visibility_lock_and_reorder() {
    let mut bridge = AubrietaGuiBridge::new();
    bridge.new_document("Layers Test").expect("doc");
    let surface_id = bridge.active_surface().unwrap();

    let mut gen = IdGenerator::new();
    let id1 = gen.next_object();
    let id2 = gen.next_object();

    bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface: surface_id,
            id: id1,
            name: "Layer 1".to_string(),
        }))
        .unwrap();

    bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface: surface_id,
            id: id2,
            name: "Layer 2".to_string(),
        }))
        .unwrap();

    let controller = LayersPanelController::new();

    // 1. Check initial layers model
    let model = controller.query_model(&bridge);
    assert_eq!(model.rows.len(), 2);
    assert_eq!(model.rows[0].id, id1);
    assert_eq!(model.rows[1].id, id2);
    assert!(model.rows[0].visible);
    assert!(!model.rows[0].locked);

    // 2. Toggle visibility of Layer 1
    controller.toggle_visibility(&mut bridge, id1).unwrap();
    let m_vis = controller.query_model(&bridge);
    assert!(!m_vis.rows[0].visible);

    // 3. Toggle lock of Layer 1
    controller.toggle_lock(&mut bridge, id1).unwrap();
    let m_lock = controller.query_model(&bridge);
    assert!(m_lock.rows[0].locked);

    // 4. Reorder: Move Layer 2 to index 0
    controller
        .reorder_row(&mut bridge, surface_id, id2, 0)
        .unwrap();
    let m_reorder = controller.query_model(&bridge);
    assert_eq!(m_reorder.rows[0].id, id2);
    assert_eq!(m_reorder.rows[1].id, id1);
}

#[test]
fn properties_panel_updates_fill_stroke_and_opacity() {
    let mut bridge = AubrietaGuiBridge::new();
    bridge.new_document("Props Test").expect("doc");
    let surface_id = bridge.active_surface().unwrap();

    let mut gen = IdGenerator::new();
    let id1 = gen.next_object();

    bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface: surface_id,
            id: id1,
            name: "Shape".to_string(),
        }))
        .unwrap();

    bridge.set_selection(vec![id1]);

    let controller = PropertiesPanelController::new();

    // Set fill
    controller
        .set_fill(&mut bridge, Some("aubrieta.indigo/500".to_string()))
        .unwrap();
    // Set stroke
    controller
        .set_stroke(&mut bridge, Some("aubrieta.gray/800".to_string()), 3.0)
        .unwrap();
    // Set opacity
    controller.set_opacity(&mut bridge, 0.75).unwrap();
    // Set bounds
    controller
        .set_bounds(&mut bridge, [100.0, 100.0, 250.0, 180.0], 0.0)
        .unwrap();

    let model = controller.query_model(&bridge);
    assert_eq!(model.fill.as_deref(), Some("aubrieta.indigo/500"));
    assert_eq!(model.stroke.as_deref(), Some("aubrieta.gray/800"));
    assert_eq!(model.stroke_width, 3.0);
    assert!((model.opacity - 0.75).abs() < 1e-6);
    assert_eq!(model.bounds, Some([100.0, 100.0, 250.0, 180.0]));
}

#[test]
fn history_panel_undo_redo_inspection() {
    let mut bridge = AubrietaGuiBridge::new();
    bridge.new_document("History Test").expect("doc");
    let surface_id = bridge.active_surface().unwrap();

    let mut gen = IdGenerator::new();
    let id1 = gen.next_object();

    bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface: surface_id,
            id: id1,
            name: "Item".to_string(),
        }))
        .unwrap();

    let controller = HistoryPanelController::new();

    // A1: new_document commits the initial canvas, so the stack holds
    // two entries (canvas + object).
    let m1 = controller.query_model(&bridge);
    assert_eq!(m1.undo_stack.len(), 2);
    assert_eq!(m1.redo_stack.len(), 0);
    assert!(m1.can_undo);
    assert!(!m1.can_redo);

    // Undo via controller (removes the object; canvas entry remains).
    controller.undo(&mut bridge).unwrap();
    let m2 = controller.query_model(&bridge);
    assert_eq!(m2.undo_stack.len(), 1);
    assert_eq!(m2.redo_stack.len(), 1);
    assert!(m2.can_undo);
    assert!(m2.can_redo);

    // Redo via controller
    controller.redo(&mut bridge).unwrap();
    let m3 = controller.query_model(&bridge);
    assert_eq!(m3.undo_stack.len(), 2);
    assert_eq!(m3.redo_stack.len(), 0);
    assert!(m3.can_undo);
    assert!(!m3.can_redo);
}

#[test]
fn properties_panel_appearance_stack_multi_fill_stroke_and_gradient() {
    use aubrieta_document::{
        AppearanceStack, BlendMode, FillItem, GradientStop, LinearGradient, StrokeItem,
    };

    let mut bridge = AubrietaGuiBridge::new();
    bridge.new_document("Appearance Test").expect("doc");
    let surface_id = bridge.active_surface().unwrap();

    let mut gen = IdGenerator::new();
    let id1 = gen.next_object();

    bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface: surface_id,
            id: id1,
            name: "GradRect".to_string(),
        }))
        .unwrap();

    bridge.set_selection(vec![id1]);

    let controller = PropertiesPanelController::new();

    let gradient = LinearGradient::new(
        [0.0, 0.0],
        [100.0, 100.0],
        vec![
            GradientStop::new(0.0, "aubrieta.indigo/500"),
            GradientStop::new(1.0, "aubrieta.emerald/500"),
        ],
    );

    let mut app = AppearanceStack::new();
    app.add_fill(FillItem::solid(1, "aubrieta.gray/100"));
    app.add_fill(FillItem::linear_gradient(2, gradient));
    app.add_stroke(StrokeItem::solid(1, "aubrieta.indigo/700", 2.5));
    app.blend_mode = BlendMode::Multiply;

    controller
        .set_appearance(&mut bridge, Some(app.clone()))
        .unwrap();

    let model = controller.query_model(&bridge);
    assert!(model.appearance.is_some());
    let res_app = model.appearance.unwrap();
    assert_eq!(res_app.fills.len(), 2);
    assert_eq!(res_app.strokes.len(), 1);
    assert_eq!(res_app.blend_mode, BlendMode::Multiply);

    // Verify undo restores previous state
    let hist_ctrl = HistoryPanelController::new();
    hist_ctrl.undo(&mut bridge).unwrap();
    let undo_model = controller.query_model(&bridge);
    assert_eq!(undo_model.appearance, None);

    // Redo restores new appearance
    hist_ctrl.redo(&mut bridge).unwrap();
    let redo_model = controller.query_model(&bridge);
    assert_eq!(redo_model.appearance, Some(app));
}

#[test]
fn layers_panel_grouping_reparenting_and_clipping_masks() {
    use aubrieta_document::ContainerRole;

    let mut bridge = AubrietaGuiBridge::new();
    bridge.new_document("Hierarchy Test").expect("doc");
    let surface_id = bridge.active_surface().unwrap();

    let mut gen = IdGenerator::new();
    let id1 = gen.next_object();
    let id2 = gen.next_object();
    let id3 = gen.next_object();

    bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface: surface_id,
            id: id1,
            name: "Shape1".to_string(),
        }))
        .unwrap();

    bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface: surface_id,
            id: id2,
            name: "Shape2".to_string(),
        }))
        .unwrap();

    bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface: surface_id,
            id: id3,
            name: "Shape3".to_string(),
        }))
        .unwrap();

    let controller = LayersPanelController::new();

    // 1. Group Shape1 and Shape2
    bridge.set_selection(vec![id1, id2]);
    controller
        .group_selection(&mut bridge, ContainerRole::Group)
        .unwrap();

    let model = controller.query_model(&bridge);
    assert_eq!(model.surfaces.len(), 1);
    // There are 4 rows now: Group, Shape1 (child), Shape2 (child), Shape3 (root)
    assert_eq!(model.rows.len(), 4);

    let grp_row = model.rows.iter().find(|r| r.is_container).unwrap();
    assert_eq!(grp_row.depth, 0);
    assert_eq!(grp_row.children_count, 2);
    assert_eq!(grp_row.role, Some(ContainerRole::Group));
    let grp_id = grp_row.id;

    // Check children depth
    let c1_row = model.rows.iter().find(|r| r.id == id1).unwrap();
    assert_eq!(c1_row.depth, 1);
    assert_eq!(c1_row.parent_id, Some(grp_id));

    let c2_row = model.rows.iter().find(|r| r.id == id2).unwrap();
    assert_eq!(c2_row.depth, 1);
    assert_eq!(c2_row.parent_id, Some(grp_id));

    let c3_row = model.rows.iter().find(|r| r.id == id3).unwrap();
    assert_eq!(c3_row.depth, 0);
    assert_eq!(c3_row.parent_id, None);

    // 2. Reparent Shape3 into Group
    controller
        .reparent_row(&mut bridge, id3, Some(grp_id), 0)
        .unwrap();
    let reparented_model = controller.query_model(&bridge);
    let c3_reparented = reparented_model.rows.iter().find(|r| r.id == id3).unwrap();
    assert_eq!(c3_reparented.depth, 1);
    assert_eq!(c3_reparented.parent_id, Some(grp_id));

    // 3. Ungroup
    bridge.set_selection(vec![grp_id]);
    controller.ungroup_selection(&mut bridge).unwrap();
    let ungrouped_model = controller.query_model(&bridge);
    assert_eq!(ungrouped_model.rows.len(), 3);
    for r in &ungrouped_model.rows {
        assert_eq!(r.depth, 0);
        assert_eq!(r.parent_id, None);
    }

    // 4. Create clipping mask between Shape1 and Shape2
    bridge.set_selection(vec![id1, id2]);
    controller.create_clipping_mask(&mut bridge).unwrap();
    let clip_model = controller.query_model(&bridge);
    let clip_grp = clip_model
        .rows
        .iter()
        .find(|r| r.role == Some(ContainerRole::ClipGroup))
        .unwrap();
    assert_eq!(clip_grp.depth, 0);
    let mask_row = clip_model.rows.iter().find(|r| r.id == id1).unwrap();
    assert!(mask_row.is_clip_mask);
    let content_row = clip_model.rows.iter().find(|r| r.id == id2).unwrap();
    assert_eq!(content_row.clip_mask_id, Some(id1));

    // 5. Release clipping mask
    controller
        .release_clipping_mask(&mut bridge, clip_grp.id)
        .unwrap();
    let released_model = controller.query_model(&bridge);
    assert_eq!(released_model.rows.len(), 3);
    let m_rel = released_model.rows.iter().find(|r| r.id == id1).unwrap();
    assert!(!m_rel.is_clip_mask);
}

#[test]
fn multi_surface_artboards_bleed_margins_guides_and_transfer() {
    use aubrieta_document::{Bleed, Guide, GuideOrientation, Margins};

    let mut bridge = AubrietaGuiBridge::new();
    bridge.new_document("Multi-Surface Test").expect("doc");
    let s1 = bridge.active_surface().unwrap();

    let s2 = aubrieta_foundation::SurfaceId::new(99);

    // Add second surface through the command lane (A6).
    bridge
        .submit_command(CommandRequest::new(Command::CreateSurface {
            id: s2,
            name: "Artboard 2".to_string(),
        }))
        .expect("second surface");

    let layers_ctrl = LayersPanelController::new();
    let props_ctrl = PropertiesPanelController::new();

    // Configure Surface 1 geometry and layout
    layers_ctrl
        .set_surface_geometry(&mut bridge, s1, [0.0, 0.0], [800.0, 600.0])
        .unwrap();
    layers_ctrl
        .set_surface_bleed(&mut bridge, s1, Bleed::uniform(9.0))
        .unwrap();
    layers_ctrl
        .set_surface_margins(&mut bridge, s1, Margins::uniform(36.0))
        .unwrap();
    layers_ctrl
        .add_guide(
            &mut bridge,
            s1,
            Guide::new(10, GuideOrientation::Horizontal, 150.0),
        )
        .unwrap();

    // Configure Surface 2 geometry
    layers_ctrl
        .set_surface_geometry(&mut bridge, s2, [1000.0, 0.0], [1200.0, 800.0])
        .unwrap();

    // Verify properties panel empty selection inspects active surface (s1)
    let empty_props = props_ctrl.query_model(&bridge);
    assert!(empty_props.selection_empty);
    let active_surf = empty_props.active_surface.unwrap();
    assert_eq!(active_surf.id, s1);
    assert_eq!(active_surf.dimensions, [800.0, 600.0]);
    assert_eq!(active_surf.bleed, Bleed::uniform(9.0));
    assert_eq!(active_surf.margins, Margins::uniform(36.0));
    assert_eq!(active_surf.guide_count, 1);

    // Create an object on Surface 1 at (1050, 50)
    let obj_id = bridge.next_object_id().unwrap();
    bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface: s1,
            id: obj_id,
            name: "Card".to_string(),
        }))
        .unwrap();
    bridge
        .set_bounds(obj_id, Some([1050.0, 50.0, 100.0, 100.0]), 0.0)
        .unwrap();

    // Move object from Surface 1 (origin 0,0) to Surface 2 (origin 1000, 0)
    layers_ctrl
        .move_row_to_surface(&mut bridge, obj_id, s2)
        .unwrap();

    // Verify object now resides on Surface 2 with coordinate compensation to (50, 50)
    let l_model = layers_ctrl.query_model(&bridge);
    assert_eq!(l_model.surfaces.len(), 2);
    let moved_row = l_model.rows.iter().find(|r| r.id == obj_id).unwrap();
    assert_eq!(moved_row.surface_id, s2);
    assert_eq!(moved_row.bounds, Some([50.0, 50.0, 100.0, 100.0]));

    // Undo moving to surface
    bridge.undo().unwrap();
    let undone_model = layers_ctrl.query_model(&bridge);
    let restored_row = undone_model.rows.iter().find(|r| r.id == obj_id).unwrap();
    assert_eq!(restored_row.surface_id, s1);
    assert_eq!(restored_row.bounds, Some([1050.0, 50.0, 100.0, 100.0]));

    // Remove guide
    layers_ctrl.remove_guide(&mut bridge, s1, 10).unwrap();
    let props_after_guide = props_ctrl.query_model(&bridge);
    assert_eq!(props_after_guide.active_surface.unwrap().guide_count, 0);
}

#[test]
fn data_merge_panel_import_bind_preflight_and_materialize() {
    use aubrieta_document::{
        BindingId, DataBinding, DataSourceId, MissingValuePolicy, TargetProperty, ValueFormatter,
    };

    let mut bridge = AubrietaGuiBridge::new();
    bridge.new_document("Data Merge UI Test").expect("doc");
    let surface = bridge.active_surface().unwrap();

    let obj_id = bridge.next_object_id().unwrap();
    bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface,
            id: obj_id,
            name: "Default Card".to_string(),
        }))
        .unwrap();

    let controller = DataMergePanelController::new();

    // 1. Import CSV
    let csv = "ID,Name,Level\n101,Ada Lovelace,Senior\n102,Alan Turing,Principal";
    let ds_id = DataSourceId::new(1);
    controller
        .import_delimited(&mut bridge, ds_id, "scientists.csv", csv, ',')
        .unwrap();

    // Query model
    let model = controller.query_model(&bridge);
    assert_eq!(model.sources.len(), 1);
    assert_eq!(model.sources[0].name, "scientists.csv");
    assert_eq!(model.sources[0].record_count, 2);
    assert_eq!(model.sources[0].field_count, 3);

    let name_field = model.sources[0]
        .fields
        .iter()
        .find(|f| f.name == "Name")
        .unwrap();

    // 2. Preflight before binding: should report unbound fields
    let findings_pre = controller.preflight(&bridge, ds_id).unwrap();
    assert_eq!(findings_pre.len(), 3); // 3 unbound fields

    // 3. Add binding: "Name" -> obj_id TextContent
    let binding = DataBinding {
        id: BindingId::new(1),
        source_id: ds_id,
        field_id: name_field.id,
        target_object: obj_id,
        target_property: TargetProperty::TextContent,
        formatter: ValueFormatter::Prefix("Dr. ".to_string()),
        missing_policy: MissingValuePolicy::Skip,
    };
    controller.add_binding(&mut bridge, binding).unwrap();

    let model_bound = controller.query_model(&bridge);
    assert_eq!(model_bound.bindings.len(), 1);
    assert_eq!(model_bound.bindings[0].field_name, "Name");

    // 4. Materialize merge: creates 2 new artboards
    controller.materialize(&mut bridge, ds_id, surface).unwrap();

    let layers_ctrl = LayersPanelController::new();
    let l_model = layers_ctrl.query_model(&bridge);
    assert_eq!(l_model.surfaces.len(), 3); // Template + 2 generated
    assert!(l_model.rows.iter().any(|r| r.name == "Dr. Ada Lovelace"));
    assert!(l_model.rows.iter().any(|r| r.name == "Dr. Alan Turing"));

    // 5. Undo materialization
    bridge.undo().unwrap();
    let l_model_undone = layers_ctrl.query_model(&bridge);
    assert_eq!(l_model_undone.surfaces.len(), 1);

    // 6. Remove data source
    controller.remove_source(&mut bridge, ds_id).unwrap();
    let model_empty = controller.query_model(&bridge);
    assert!(model_empty.sources.is_empty());
    assert!(model_empty.bindings.is_empty());
}
