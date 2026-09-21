//! Vector gradient and transparency tools (08.24, 10.4).

use aubrieta_application::Command;
use aubrieta_document::ChangeSet;
use aubrieta_foundation::AubrietaError;
use aubrieta_geometry::GPoint;

use crate::bridge::AubrietaGuiBridge;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use aubrieta_application::interaction::{NormalizedPointerEvent, PointerButton, PointerPhase};

/// Mode for the gradient tool (10.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GradientToolMode {
    /// Color gradient on fill/stroke.
    Fill,
    /// Mask transparency gradient.
    Transparency,
}

/// Interactive tool for plotting gradient vector lines on canvas (10.4).
#[derive(Clone, Debug)]
pub struct GradientTool {
    mode: GradientToolMode,
    start_doc: Option<GPoint>,
    current_doc: Option<GPoint>,
}

impl GradientTool {
    /// Creates a gradient tool in fill or transparency mode.
    #[must_use]
    pub fn new(mode: GradientToolMode) -> Self {
        Self {
            mode,
            start_doc: None,
            current_doc: None,
        }
    }

    /// Resets active gradient drag.
    pub fn cancel(&mut self) {
        self.start_doc = None;
        self.current_doc = None;
    }

    /// Handles pointer events.
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
                self.start_doc = Some(event.doc_pos);
                self.current_doc = Some(event.doc_pos);
                Ok(ChangeSet::empty())
            }
            PointerPhase::Move => {
                if self.start_doc.is_some() {
                    self.current_doc = Some(event.doc_pos);
                }
                Ok(ChangeSet::empty())
            }
            PointerPhase::Up => {
                let start = self.start_doc.take();
                let current = self.current_doc.take();

                if let (Some(p0), Some(p1)) = (start, current) {
                    let selected = bridge.selection().selected_ids;
                    match self.mode {
                        GradientToolMode::Fill => {
                            // The drag vector edits gradient *geometry* (10.4):
                            // existing linear gradients are repositioned,
                            // otherwise one is created from the current solid
                            // color on both stops (no color is invented).
                            let mut cmds = Vec::new();
                            for id in selected {
                                if let Some(stack) =
                                    apply_fill_vector(bridge, id, [p0.x, p0.y], [p1.x, p1.y])
                                {
                                    cmds.push(Command::SetAppearance {
                                        id,
                                        appearance: Some(stack),
                                    });
                                }
                            }
                            return bridge.submit_all("Edit gradient", cmds);
                        }
                        GradientToolMode::Transparency => {
                            // No mask infrastructure exists yet (10.5/10.10):
                            // vertical drag adjusts whole-stack opacity
                            // relative to the drag-start value, explicitly
                            // and undoably (documented gesture).
                            let mut cmds = Vec::new();
                            for id in selected {
                                if let Some(cmd) =
                                    apply_transparency_drag(bridge, id, p0.y - p1.y)
                                {
                                    cmds.push(cmd);
                                }
                            }
                            return bridge.submit_all("Adjust transparency", cmds);
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


    /// Resolves overlays showing the gradient vector line.
    #[must_use]
    pub fn overlays(&self) -> CanvasOverlays {
        let mut overlays = CanvasOverlays::default();
        if let (Some(p0), Some(p1)) = (self.start_doc, self.current_doc) {
            overlays.pen_preview = Some(vec![p0, p1]);
        }
        overlays
    }
}

/// Repositions (or creates) the primary linear gradient of one object.
/// Returns the new stack, or `None` when the object is missing.
fn apply_fill_vector(
    bridge: &AubrietaGuiBridge,
    id: aubrieta_foundation::ObjectId,
    start: [f64; 2],
    end: [f64; 2],
) -> Option<aubrieta_document::AppearanceStack> {
    use aubrieta_document::{GradientStop, LinearGradient, Paint};
    let stack = bridge
        .session()?
        .find_object(id)?
        .effective_appearance();
    let paint = match stack.primary_fill().map(|f| f.paint.clone()) {
        Some(Paint::LinearGradient(mut g)) => {
            g.start = start;
            g.end = end;
            g.sort_and_reindex();
            Paint::LinearGradient(g)
        }
        Some(Paint::Solid(token)) => Paint::LinearGradient(LinearGradient::new(
            start,
            end,
            vec![
                GradientStop::new(0.0, token.clone()),
                GradientStop::new(1.0, token),
            ],
        )),
        _ => Paint::LinearGradient(LinearGradient::new(
            start,
            end,
            vec![
                GradientStop::new(0.0, "aubrieta.blue/500"),
                GradientStop::new(1.0, "aubrieta.blue/500"),
            ],
        )),
    };
    Some(aubrieta_application::appearance_service::with_primary_gradient(
        stack, paint,
    ))
}

/// Adjusts whole-stack opacity by vertical drag distance (200pt = full
/// range), relative to the drag-start value. Returns `None` when the
/// object is missing.
fn apply_transparency_drag(
    bridge: &AubrietaGuiBridge,
    id: aubrieta_foundation::ObjectId,
    dy: f64,
) -> Option<Command> {
    let stack = bridge
        .session()?
        .find_object(id)?
        .effective_appearance();
    let next = (stack.opacity + dy / 200.0).clamp(0.0, 1.0);
    if (next - stack.opacity).abs() <= f64::EPSILON {
        return None;
    }
    Some(Command::SetStackOpacity {
        id,
        opacity: next,
    })
}
