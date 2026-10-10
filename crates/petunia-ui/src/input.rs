//! User input events, keyboard shortcuts, and active tool enumeration.

use crate::tooltips::Tooltip;
use petunia_core::Point;

/// Active studio creative tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToolKind {
    #[default]
    Select,
    Pen,
    NodeEdit,
    Rectangle,
    Ellipse,
    Zoom,
}

/// Pointer interaction events (mouse or stylus tablet).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PointerEvent {
    Down { position: Point, pressure: f32 },
    Move { position: Point, pressure: f32 },
    Up { position: Point },
}

/// Top-level user action intents.
#[derive(Debug, Clone, PartialEq)]
pub enum UserAction {
    /// Switch the active tool.
    SelectTool(ToolKind),
    /// Pointer event on the canvas.
    Pointer(PointerEvent),
    /// Undo one transaction.
    Undo,
    /// Redo one transaction.
    Redo,
    /// Smart Delete on the current node sub-selection (ADR-0012 D7).
    SmartDelete(petunia_engine::geometry::SmartDeleteMode),
    /// Move the selection by document units, independent of zoom.
    Nudge { dx: f64, dy: f64 },
    /// Text of a hover or focus event, for the tooltip layer.
    HoverTooltip(Tooltip),
}
