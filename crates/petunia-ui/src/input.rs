//! User input events, keyboard shortcuts, and active tool enumeration.

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
    SelectTool(ToolKind),
    Pointer(PointerEvent),
    Undo,
    Redo,
}
