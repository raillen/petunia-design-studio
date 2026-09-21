//! Input normalization and semantic modifiers (09.27, 10.1).

use petunia_design_geometry::GPoint;
use serde::{Deserialize, Serialize};

/// Pointer gesture phase.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PointerPhase {
    /// Initial pointer button down.
    Down,
    /// Pointer moving (drag or hover).
    Move,
    /// Pointer button released.
    Up,
    /// Gesture cancelled by window loss, Esc, or system gesture.
    Cancel,
}

/// Primary pointer buttons.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PointerButton {
    /// Left button or primary touch/pen tip.
    Primary,
    /// Middle wheel click.
    Middle,
    /// Right button or secondary stylus button.
    Secondary,
}

/// High-level semantic modifier intents decoupled from platform-specific keys (10.1).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticModifiers {
    /// Constrain aspect ratio, angle (15/45 deg increments), or axis-alignment (e.g. Shift).
    pub constrain: bool,
    /// Expand transformation symmetrically from selection or shape center (e.g. Alt/Option).
    pub from_center: bool,
    /// Duplicate selected objects during drag operation (e.g. Alt-drag).
    pub duplicate: bool,
    /// Temporarily bypass magnetic snapping engine (e.g. Ctrl/Cmd).
    pub disable_snap: bool,
    /// Micro-nudge / high-precision fractional adjustment.
    pub fine_adjust: bool,
}

/// Normalized pointer event delivered to interactive tools.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NormalizedPointerEvent {
    /// Interaction phase.
    pub phase: PointerPhase,
    /// Active pointer button.
    pub button: PointerButton,
    /// Position in screen pixels relative to canvas top-left.
    pub screen_pos: GPoint,
    /// Evaluated position in document points via viewport camera.
    pub doc_pos: GPoint,
    /// Normalized semantic modifier intents.
    pub modifiers: SemanticModifiers,
    /// Stylus pressure factor in `[0.0, 1.0]` (defaults to 1.0 for mouse).
    pub pressure: f64,
}

impl NormalizedPointerEvent {
    /// Creates a normalized pointer event.
    #[must_use]
    pub fn new(
        phase: PointerPhase,
        button: PointerButton,
        screen_pos: GPoint,
        doc_pos: GPoint,
        modifiers: SemanticModifiers,
    ) -> Self {
        Self {
            phase,
            button,
            screen_pos,
            doc_pos,
            modifiers,
            pressure: 1.0,
        }
    }
}
