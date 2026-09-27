//! Eyedropper and style sampling tools (08.24, 09.25, 10.4, TOOLS_DECISIONS Batch 8).
//!
//! Color mode samples the full paint pair (primary fill paint — solid token
//! or gradient kept as-is — plus primary stroke paint and width) into every
//! selected object with one undo entry. Style mode copies the whole
//! appearance stack. Sampling ignores locked objects.

use petunia_design_application::Command;
use petunia_design_document::{AppearanceStack, ChangeSet, Paint};
use petunia_design_foundation::{ObjectId, PetuniaError};
use petunia_design_geometry::GPoint;

use crate::bridge::PetuniaDesignGuiBridge;
use crate::canvas::{CanvasOverlays, CursorAffordance, SnapEngine, ViewportCamera};

use petunia_design_application::interaction::{
    NormalizedPointerEvent, PointerButton, PointerPhase,
};

/// Operational mode for the picker tool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickerMode {
    /// Color eyedropper sampling fill and stroke paints.
    Color,
    /// Style picker copying AppearanceStack and properties.
    Style,
}

/// Sampled color pair: fill paint plus optional stroke paint and width.
#[derive(Clone, Debug, PartialEq)]
pub struct ColorSample {
    /// Primary fill paint of the sampled object.
    pub fill: Paint,
    /// Primary stroke paint, if the sampled object strokes.
    pub stroke: Option<Paint>,
    /// Primary stroke width, if stroking.
    pub stroke_width: Option<f64>,
}

/// Eyedropper and style sampler tool.
#[derive(Clone, Debug)]
pub struct PickerTool {
    mode: PickerMode,
    hover_doc: Option<GPoint>,
}

impl PickerTool {
    /// Creates a picker tool in color or style mode.
    #[must_use]
    pub fn new(mode: PickerMode) -> Self {
        Self {
            mode,
            hover_doc: None,
        }
    }

    /// Current mode.
    #[must_use]
    pub fn mode(&self) -> PickerMode {
        self.mode
    }

    /// Resets tool state.
    pub fn cancel(&mut self) {
        self.hover_doc = None;
    }

    /// Handles normalized pointer events.
    pub fn on_pointer_event(
        &mut self,
        event: &NormalizedPointerEvent,
        bridge: &mut PetuniaDesignGuiBridge,
        _camera: &ViewportCamera,
        _snap: &mut SnapEngine,
    ) -> Result<ChangeSet, PetuniaError> {
        if event.phase == PointerPhase::Move {
            self.hover_doc = Some(event.doc_pos);
            return Ok(ChangeSet::empty());
        }
        if event.phase == PointerPhase::Cancel {
            self.cancel();
            return Ok(ChangeSet::empty());
        }
        if event.phase != PointerPhase::Up || event.button != PointerButton::Primary {
            return Ok(ChangeSet::empty());
        }

        let pt = event.doc_pos;
        self.hover_doc = Some(pt);
        let session = match bridge.session() {
            Some(s) => s,
            None => return Ok(ChangeSet::empty()),
        };

        // Hit-test in reverse draw order (topmost first), skipping locked (F3 spatial).
        let hit = match session
            .spatial_candidates_point(pt, 0.0)
            .into_iter()
            .find_map(|id| {
                let obj = session.find_object(id)?;
                if obj.visible && !obj.locked && obj.hit_test(pt) {
                    Some(obj.clone())
                } else {
                    None
                }
            }) {
            Some(obj) => obj,
            None => return Ok(ChangeSet::empty()),
        };

        let selected_ids = bridge.selection().selected_ids;
        let mut all_cmds = Vec::new();

        match self.mode {
            PickerMode::Color => {
                let sample = color_sample(&hit);
                for sel_id in selected_ids {
                    all_cmds.extend(color_sample_commands(bridge, sel_id, &sample));
                }
            }
            PickerMode::Style => {
                let style = petunia_design_application::appearance_service::sample_style(&hit);
                for sel_id in selected_ids {
                    all_cmds.extend(
                        petunia_design_application::appearance_service::style_sample_commands(
                            sel_id, &style,
                        ),
                    );
                }
            }
        }

        if all_cmds.is_empty() {
            return Ok(ChangeSet::empty());
        }
        let label = match self.mode {
            PickerMode::Color => "Pick color",
            PickerMode::Style => "Pick style",
        };
        bridge.submit_all(label, all_cmds)
    }

    /// Resolves overlays with hover highlighting and cursor affordances.
    #[must_use]
    pub fn overlays(&self, bridge: &PetuniaDesignGuiBridge) -> CanvasOverlays {
        let mut overlays = CanvasOverlays::default();
        overlays.cursor = CursorAffordance::Crosshair;
        if let Some(pt) = self.hover_doc {
            if let Some(session) = bridge.session() {
                let hit = session
                    .spatial_candidates_point(pt, 0.0)
                    .into_iter()
                    .find(|id| {
                        session.find_object(*id).is_some_and(|obj| {
                            obj.visible && !obj.locked && obj.hit_test(pt)
                        })
                    });
                overlays.hovered_object = hit;
                if hit.is_some() {
                    overlays.cursor = CursorAffordance::Pointer;
                }
            }
        }
        overlays
    }
}

/// Samples the primary fill and stroke paints of one object.
fn color_sample(source: &petunia_design_document::DocumentObject) -> ColorSample {
    let stack = source.effective_appearance();
    let fill = stack
        .primary_fill()
        .map(|f| f.paint.clone())
        .unwrap_or(Paint::Solid(
            source
                .fill
                .clone()
                .unwrap_or_else(|| "ptnd.gray/500".to_string()),
        ));
    let (stroke, stroke_width) = stack
        .primary_stroke()
        .map(|s| (s.paint.clone(), s.width))
        .unwrap_or_else(|| {
            (
                Paint::Solid(
                    source
                        .stroke
                        .clone()
                        .unwrap_or_else(|| "ptnd.gray/500".to_string()),
                ),
                source.stroke_width,
            )
        });
    // A stroke entry always exists in the effective stack; apply it only
    // when the source visibly strokes (painted entry or legacy token).
    let strokes = source.stroke.is_some()
        || stack
            .strokes
            .iter()
            .any(|s| s.visible && !matches!(s.paint, Paint::None));
    ColorSample {
        fill,
        stroke: strokes.then_some(stroke),
        stroke_width: strokes.then_some(stroke_width),
    }
}

/// Builds one `SetAppearance` applying a color sample to a target,
/// preserving the target's stack structure (fills/strokes/effects).
fn color_sample_commands(
    bridge: &PetuniaDesignGuiBridge,
    target_id: ObjectId,
    sample: &ColorSample,
) -> Vec<Command> {
    let session = match bridge.session() {
        Some(s) => s,
        None => return Vec::new(),
    };
    let Some(target) = session.find_object(target_id) else {
        return Vec::new();
    };
    let mut stack: AppearanceStack = target.effective_appearance();
    match stack.fills.first_mut() {
        Some(first) => {
            first.paint = sample.fill.clone();
        }
        None => {
            stack.fills.push(petunia_design_document::FillItem {
                id: 1,
                paint: sample.fill.clone(),
                opacity: 1.0,
                blend_mode: petunia_design_document::BlendMode::Normal,
                visible: true,
            });
        }
    }
    if let (Some(paint), Some(width)) = (sample.stroke.clone(), sample.stroke_width) {
        match stack.strokes.iter_mut().find(|s| s.visible) {
            Some(entry) => {
                entry.paint = paint;
                entry.width = width.max(0.0);
            }
            None => {
                let nid = stack.strokes.iter().map(|s| s.id).max().unwrap_or(0) + 1;
                stack
                    .strokes
                    .push(petunia_design_document::StrokeItem::solid(
                        nid,
                        paint_name(&paint),
                        width,
                    ));
                if let Some(entry) = stack.strokes.last_mut() {
                    entry.paint = paint;
                }
            }
        }
    }
    vec![Command::SetAppearance {
        id: target_id,
        appearance: Some(stack),
    }]
}

/// Display name for a paint (solid tokens pass through; others describe).
fn paint_name(paint: &Paint) -> String {
    match paint {
        Paint::Solid(token) => token.clone(),
        Paint::LinearGradient(_) => "gradient:linear".to_string(),
        Paint::RadialGradient(_) => "gradient:radial".to_string(),
        Paint::None => "none".to_string(),
    }
}
