//! Interactive 4-corner perspective warp tool (10.8, TOOLS_DECISIONS Batch 14).
//!
//! Drags one corner of the selection quad at a time; on release every
//! selected object receives the absolute quad as a live `Perspective`
//! modifier in ONE undo entry. The quad is absolute document space: moving
//! an object afterwards does not move its warp (documented V1 limit;
//! bounds-anchored relative quads stay future work).

use petunia_design_application::Command;
use petunia_design_document::ChangeSet;
use petunia_design_foundation::PetuniaError;
use petunia_design_geometry::GPoint;

use crate::bridge::PetuniaDesignGuiBridge;
use crate::canvas::{
    CanvasOverlays, SelectionHandle, SelectionHandleKind, SnapEngine, ViewportCamera,
};

use petunia_design_application::interaction::{NormalizedPointerEvent, PointerButton, PointerPhase};

/// Corner hit radius in screen pixels.
const CORNER_HIT_PX: f64 = 14.0;

/// Pending corner drag.
#[derive(Clone, Debug)]
struct CornerDrag {
    corner: usize,
    current_doc: GPoint,
}

/// Interactive perspective warp tool.
#[derive(Clone, Debug, Default)]
pub struct PerspectiveTool {
    drag: Option<CornerDrag>,
}

impl PerspectiveTool {
    /// Creates a fresh perspective tool.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Cancels any in-flight drag (selection is kept).
    pub fn cancel(&mut self) {
        self.drag = None;
    }

    /// True while a corner drag is in flight.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.drag.is_some()
    }

    /// Handles normalized pointer events.
    pub fn on_pointer_event(
        &mut self,
        event: &NormalizedPointerEvent,
        bridge: &mut PetuniaDesignGuiBridge,
        camera: &ViewportCamera,
        snap: &mut SnapEngine,
    ) -> Result<ChangeSet, PetuniaError> {
        match event.phase {
            PointerPhase::Down => self.on_down(event, bridge, camera, snap),
            PointerPhase::Move => self.on_move(event),
            PointerPhase::Up => self.on_up(event, bridge),
            PointerPhase::Cancel => {
                self.cancel();
                snap.reset_hysteresis();
                Ok(ChangeSet::empty())
            }
        }
    }

    fn on_down(
        &mut self,
        event: &NormalizedPointerEvent,
        bridge: &mut PetuniaDesignGuiBridge,
        camera: &ViewportCamera,
        snap: &mut SnapEngine,
    ) -> Result<ChangeSet, PetuniaError> {
        if event.button != PointerButton::Primary {
            return Ok(ChangeSet::empty());
        }
        snap.reset_hysteresis();
        if let Some(quad) = current_quad(bridge) {
            let tol = CORNER_HIT_PX / camera.zoom.max(0.1);
            for (i, corner) in quad.iter().enumerate() {
                if corner.distance_to(event.doc_pos) <= tol {
                    self.drag = Some(CornerDrag {
                        corner: i,
                        current_doc: event.doc_pos,
                    });
                    return Ok(ChangeSet::empty());
                }
            }
        }
        Ok(ChangeSet::empty())
    }

    fn on_move(&mut self, event: &NormalizedPointerEvent) -> Result<ChangeSet, PetuniaError> {
        if let Some(drag) = &mut self.drag {
            drag.current_doc = event.doc_pos;
        }
        Ok(ChangeSet::empty())
    }

    fn on_up(
        &mut self,
        event: &NormalizedPointerEvent,
        bridge: &mut PetuniaDesignGuiBridge,
    ) -> Result<ChangeSet, PetuniaError> {
        let Some(drag) = self.drag.take() else {
            return Ok(ChangeSet::empty());
        };
        let Some(mut quad) = current_quad(bridge) else {
            return Ok(ChangeSet::empty());
        };
        quad[drag.corner] = event.doc_pos;
        let quad_array = [
            [quad[0].x, quad[0].y],
            [quad[1].x, quad[1].y],
            [quad[2].x, quad[2].y],
            [quad[3].x, quad[3].y],
        ];
        let mut cmds = Vec::new();
        for id in bridge.selection().selected_ids.clone() {
            cmds.push(Command::SetPerspective {
                id,
                quad: quad_array,
            });
        }
        if cmds.is_empty() {
            return Ok(ChangeSet::empty());
        }
        bridge.submit_all("Perspective warp", cmds)
    }

    /// Resolves overlays: quad corner handles plus the pending warp outline.
    #[must_use]
    pub fn overlays(
        &self,
        bridge: &PetuniaDesignGuiBridge,
        camera: &ViewportCamera,
    ) -> CanvasOverlays {
        let mut overlays = CanvasOverlays::default();
        let Some(quad) = current_quad(bridge) else {
            return overlays;
        };
        let kinds = [
            SelectionHandleKind::TopLeft,
            SelectionHandleKind::TopRight,
            SelectionHandleKind::BottomRight,
            SelectionHandleKind::BottomLeft,
        ];
        for (pt, kind) in quad.iter().zip(kinds) {
            let screen = camera.doc_to_screen(*pt);
            overlays.handles.push(SelectionHandle {
                kind,
                doc_point: *pt,
                screen_hit_box: petunia_design_geometry::GRect::new(
                    screen.x - 7.0,
                    screen.y - 7.0,
                    screen.x + 7.0,
                    screen.y + 7.0,
                ),
            });
        }
        // Pending warp preview of the first selected object.
        if let Some(drag) = &self.drag {
            let mut pending = quad;
            pending[drag.corner] = drag.current_doc;
            if let Some(outline) = pending_outline(bridge, pending) {
                overlays.region_preview = Some(outline);
            }
        }
        overlays
    }
}

/// Current quad: the live quad of the first selected object carrying one,
/// else the combined evaluated-bounds corners of the selection.
fn current_quad(bridge: &PetuniaDesignGuiBridge) -> Option<[GPoint; 4]> {
    let session = bridge.session()?;
    if session.selection.selected_ids.is_empty() {
        return None;
    }
    for id in &session.selection.selected_ids {
        if let Some(obj) = session.find_object(*id) {
            for modifier in &obj.modifiers {
                if !modifier.enabled {
                    continue;
                }
                if let petunia_design_document::ModifierKind::Perspective { quad } =
                    &modifier.kind
                {
                    return Some(quad.map(|[x, y]| GPoint::new(x, y)));
                }
            }
        }
    }
    // Fall back to combined evaluated bounds.
    let mut x0 = f64::MAX;
    let mut y0 = f64::MAX;
    let mut x1 = f64::MIN;
    let mut y1 = f64::MIN;
    let mut any = false;
    for id in &session.selection.selected_ids {
        if let Some(obj) = session.find_object(*id) {
            if let Some([x, y, w, h]) = obj.evaluated_bounds() {
                any = true;
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x + w);
                y1 = y1.max(y + h);
            }
        }
    }
    if !any {
        return None;
    }
    Some([
        GPoint::new(x0, y0),
        GPoint::new(x1, y0),
        GPoint::new(x1, y1),
        GPoint::new(x0, y1),
    ])
}

/// Pending warped outline of the first selected object, flattened.
fn pending_outline(bridge: &PetuniaDesignGuiBridge, quad: [GPoint; 4]) -> Option<Vec<GPoint>> {
    let session = bridge.session()?;
    let id = *session.selection.selected_ids.first()?;
    let obj = session.find_object(id)?;
    let base = obj.to_path();
    if base.is_empty() {
        return None;
    }
    let warped = petunia_design_geometry::warp_path_to_quad(&base, quad, 0.5)?;
    let flat: Vec<GPoint> = warped.to_polygons(0.5).into_iter().flatten().collect();
    if flat.len() >= 2 {
        Some(flat)
    } else {
        None
    }
}
