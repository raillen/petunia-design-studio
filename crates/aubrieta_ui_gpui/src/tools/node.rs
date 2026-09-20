//! Node editing and direct path manipulation tool (10.2).

use aubrieta_application::{Command, CommandRequest};
use aubrieta_document::ChangeSet;
use aubrieta_foundation::{AubrietaError, ObjectId};
use aubrieta_geometry::GPoint;

use crate::bridge::AubrietaGuiBridge;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use super::input::{NormalizedPointerEvent, PointerButton, PointerPhase};

/// Direct selection and node editing tool (10.2).
#[derive(Clone, Debug, Default)]
pub struct NodeTool {
    active_object: Option<ObjectId>,
    active_node_idx: Option<usize>,
    is_dragging: bool,
    drag_start: GPoint,
}

impl NodeTool {
    /// Creates a fresh Node tool.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Resets active node selection.
    pub fn cancel(&mut self) {
        self.active_node_idx = None;
        self.is_dragging = false;
    }

    /// Handles normalized pointer events.
    pub fn on_pointer_event(
        &mut self,
        event: &NormalizedPointerEvent,
        bridge: &mut AubrietaGuiBridge,
        _camera: &ViewportCamera,
        _snap: &mut SnapEngine,
    ) -> Result<ChangeSet, AubrietaError> {
        match event.phase {
            PointerPhase::Down => {
                if event.button != PointerButton::Primary {
                    return Ok(ChangeSet::empty());
                }
                let sel_vm = bridge.selection();
                if let Some(&first_id) = sel_vm.selected_ids.first() {
                    self.active_object = Some(first_id);
                    self.active_node_idx = Some(0);
                    self.is_dragging = true;
                    self.drag_start = event.doc_pos;
                }
                Ok(ChangeSet::empty())
            }
            PointerPhase::Move => Ok(ChangeSet::empty()),
            PointerPhase::Up => {
                if self.is_dragging {
                    self.is_dragging = false;
                    let dx = event.doc_pos.x - self.drag_start.x;
                    let dy = event.doc_pos.y - self.drag_start.y;
                    if let Some(obj_id) = self.active_object {
                        if let Some(session) = bridge.session() {
                            if let Some(obj) = session.document.find_object(obj_id) {
                                if let Some([x, y, w, h]) = obj.bounds {
                                    let cmd = CommandRequest::new(Command::SetBounds {
                                        id: obj_id,
                                        bounds: Some([x + dx, y + dy, w, h]),
                                        rotation: obj.rotation,
                                    });
                                    return bridge.submit_command(cmd);
                                }
                            }
                        }
                    }
                }
                Ok(ChangeSet::empty())
            }
            PointerPhase::Cancel => {
                self.cancel();
                Ok(ChangeSet::empty())
            }
        }
    }

    /// Resolves overlays for the Node tool.
    #[must_use]
    pub fn overlays(&self) -> CanvasOverlays {
        CanvasOverlays::default()
    }
}
