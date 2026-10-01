//! Transaction/cancellation/selection regression sources, execution deferred.
use petunia_design_application::{
    raster_edit::{RasterBrush, RasterStroke},
    Command, CommandRequest, DocumentSession, SelectionMode, SelectionShape,
};
use petunia_design_document::{Document, DocumentMutator, ShapeKind};
use petunia_design_foundation::{ObjectId, SurfaceId};
use petunia_design_geometry::GPoint;
use std::sync::Arc;
fn session() -> DocumentSession {
    let mut document = Document::new();
    let mut m = DocumentMutator::new(&mut document);
    m.add_surface(SurfaceId::new(1), "Page").unwrap();
    m.set_surface_geometry(SurfaceId::new(1), [0.0, 0.0], [32.0, 32.0])
        .unwrap();
    DocumentSession::with_document("Paint", document)
}
fn brush() -> RasterBrush {
    RasterBrush {
        radius: 3.0,
        hardness: 1.0,
        flow: 1.0,
        opacity: 1.0,
        color: [1.0, 0.0, 0.0, 1.0],
        erase: false,
    }
}
fn pixel(s: &DocumentSession, x: i64, y: i64) -> [f32; 4] {
    let Some(ShapeKind::Raster { layer }) = &s.document().surfaces()[0].objects()[0].shape else {
        panic!("raster expected")
    };
    layer.pixel(x, y)
}
#[test]
fn click_and_drag_publish_one_reversible_creation() {
    let mut s = session();
    let before = s.document().clone();
    let mut stroke = RasterStroke::begin(&s, brush()).unwrap();
    stroke.sample(GPoint::new(10.5, 10.5), 1.0).unwrap();
    stroke.sample(GPoint::new(20.5, 10.5), 1.0).unwrap();
    assert_eq!(s.document(), &before);
    stroke.commit(&mut s).unwrap();
    assert_eq!(s.current_revision(), 1);
    assert_eq!(pixel(&s, 15, 10), [1.0, 0.0, 0.0, 1.0]);
    let painted = s.document().clone();
    assert!(s.undo().unwrap());
    assert_eq!(s.document(), &before);
    assert!(s.redo().unwrap());
    assert_eq!(s.document(), &painted);
}
#[test]
fn dropping_a_real_pixel_draft_leaves_no_artwork_or_undo() {
    let mut s = session();
    let before = s.document().clone();
    let mut stroke = RasterStroke::begin(&s, brush()).unwrap();
    stroke.sample(GPoint::new(8.0, 8.0), 1.0).unwrap();
    drop(stroke);
    assert_eq!(s.document(), &before);
    assert!(!s.undo().unwrap());
    assert_eq!(s.current_revision(), 0);
}
#[test]
fn same_revision_other_session_cannot_accept_a_draft() {
    let s = session();
    let mut other = session();
    let before = other.document().clone();
    let mut draft = RasterStroke::begin(&s, brush()).unwrap();
    draft.sample(GPoint::new(8.0, 8.0), 1.0).unwrap();
    assert!(draft.commit(&mut other).is_err());
    assert_eq!(other.document(), &before);
}
#[test]
fn intervening_edit_rejects_pixels_without_reverting_that_edit() {
    let mut s = session();
    let mut draft = RasterStroke::begin(&s, brush()).unwrap();
    draft.sample(GPoint::new(8.0, 8.0), 1.0).unwrap();
    s.execute_command(CommandRequest::new(Command::CreateObject {
        surface: SurfaceId::new(1),
        id: ObjectId::new(2),
        name: "Other".into(),
    }))
    .unwrap();
    let before = s.document().clone();
    assert!(draft.commit(&mut s).is_err());
    assert_eq!(s.document(), &before);
}
#[test]
fn active_empty_selection_prevents_layer_creation() {
    let mut s = session();
    let rectangle = SelectionShape::Rect {
        x0: 0.0,
        y0: 0.0,
        x1: 10.0,
        y1: 10.0,
    };
    s.raster_selection
        .combine(&rectangle, SelectionMode::Replace);
    s.raster_selection
        .combine(&rectangle, SelectionMode::Subtract);
    assert!(s.raster_selection.is_active());
    assert!(s.raster_selection.is_empty());
    let mut draft = RasterStroke::begin(&s, brush()).unwrap();
    draft.sample(GPoint::new(8.0, 8.0), 1.0).unwrap();
    assert!(draft.commit(&mut s).unwrap().is_empty());
    assert_eq!(s.current_revision(), 0);
}
#[test]
fn selection_clips_paint_and_deselect_restores_full_coverage() {
    let mut s = session();
    s.raster_selection.combine(
        &SelectionShape::Rect {
            x0: 0.0,
            y0: 0.0,
            x1: 12.0,
            y1: 32.0,
        },
        SelectionMode::Replace,
    );
    let mut draft = RasterStroke::begin(&s, brush()).unwrap();
    draft.sample(GPoint::new(11.5, 10.5), 1.0).unwrap();
    draft.commit(&mut s).unwrap();
    assert_eq!(pixel(&s, 11, 10)[3], 1.0);
    assert_eq!(pixel(&s, 13, 10)[3], 0.0);
    s.raster_selection.clear();
    let mut draft = RasterStroke::begin(&s, brush()).unwrap();
    draft.sample(GPoint::new(15.5, 10.5), 1.0).unwrap();
    draft.commit(&mut s).unwrap();
    assert_eq!(pixel(&s, 15, 10)[3], 1.0);
}
#[test]
fn eraser_changes_pixels_on_the_selected_layer_and_undo_restores_them() {
    let mut s = session();
    let mut paint = RasterStroke::begin(&s, brush()).unwrap();
    paint.sample(GPoint::new(8.5, 8.5), 1.0).unwrap();
    paint.commit(&mut s).unwrap();
    let before = s.document().clone();
    let mut b = brush();
    b.erase = true;
    let mut erase = RasterStroke::begin(&s, b).unwrap();
    erase.sample(GPoint::new(8.5, 8.5), 1.0).unwrap();
    erase.commit(&mut s).unwrap();
    assert_eq!(pixel(&s, 8, 8)[3], 0.0);
    assert!(s.undo().unwrap());
    assert_eq!(s.document(), &before);
}
#[test]
fn locked_pixel_target_cannot_start_a_gesture() {
    let mut s = session();
    let mut paint = RasterStroke::begin(&s, brush()).unwrap();
    paint.sample(GPoint::new(8.5, 8.5), 1.0).unwrap();
    paint.commit(&mut s).unwrap();
    let id = s.selection.selected_ids[0];
    s.execute_command(CommandRequest::new(Command::SetLocked { id, locked: true }))
        .unwrap();
    assert!(RasterStroke::begin(&s, brush()).is_err());
}
#[test]
fn fill_is_four_connected_and_one_undo_entry() {
    let mut s = session();
    let mut draft = RasterStroke::begin(&s, brush()).unwrap();
    draft
        .flood_fill(GPoint::new(1.0, 1.0), 0.0, &|| false)
        .unwrap();
    draft.commit(&mut s).unwrap();
    assert_eq!(pixel(&s, 0, 0), [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(pixel(&s, 31, 31), [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(s.current_revision(), 1);
    assert!(s.undo().unwrap());
    assert!(s.document().surfaces()[0].objects().is_empty());
}
#[test]
fn cancelled_fill_and_outside_seed_leave_canonical_state_untouched() {
    let mut s = session();
    let before = s.document().clone();
    let mut draft = RasterStroke::begin(&s, brush()).unwrap();
    assert!(draft
        .flood_fill(GPoint::new(1.0, 1.0), 0.0, &|| true)
        .is_err());
    drop(draft);
    let mut draft = RasterStroke::begin(&s, brush()).unwrap();
    draft
        .flood_fill(GPoint::new(-1.0, 1.0), 0.0, &|| false)
        .unwrap();
    assert!(draft.commit(&mut s).unwrap().is_empty());
    assert_eq!(s.document(), &before);
}
#[test]
fn stroke_master_opacity_is_not_multiplied_by_dab_overlap() {
    let mut s = session();
    let mut b = brush();
    b.opacity = 0.5;
    let mut draft = RasterStroke::begin(&s, b).unwrap();
    draft.sample(GPoint::new(8.5, 8.5), 1.0).unwrap();
    draft.sample(GPoint::new(9.5, 8.5), 1.0).unwrap();
    draft.sample(GPoint::new(8.5, 8.5), 1.0).unwrap();
    draft.commit(&mut s).unwrap();
    assert!((pixel(&s, 8, 8)[3] - 0.5).abs() < 0.005);
}
#[test]
fn draft_previews_change_identity_without_changing_document_revision() {
    let s = session();
    let mut draft = RasterStroke::begin(&s, brush()).unwrap();
    let previous = draft.preview_source();
    draft.sample(GPoint::new(8.5, 8.5), 1.0).unwrap();
    let next = draft.preview_source();
    assert_ne!(previous.id(), next.id());
    assert_eq!(next.revision(), s.current_revision());
    assert!(!Arc::ptr_eq(&previous, &next));
    assert!(s.document().surfaces()[0].objects().is_empty());
}
#[test]
fn recovered_sessions_require_save_as_and_close_confirmation() {
    let s = session();
    let mut recovered = DocumentSession::with_recovered_document("Recovered", s.document().clone());
    assert!(recovered.is_dirty());
    assert!(recovered.path().is_none());
    assert_eq!(recovered.current_revision(), 0);
    assert!(!recovered.undo().unwrap());
    recovered.mark_saved();
    assert!(!recovered.is_dirty());
}
