//! # petunia-ui
//!
//! Headless-first interaction layer: contexts, selection, tools,
//! shortcuts, numeric fields and workspace state.
//!
//! Strict invariants:
//! - `#![forbid(unsafe_code)]`
//! - No Qt, QML or GUI toolkit type appears here.
//! - Mutations travel as engine transactions; never on hover.
//! - Every piece of session state is Session State, never PTND.

#![forbid(unsafe_code)]

pub mod app;
pub mod context;
pub mod error;
pub mod input;
pub mod numeric;
pub mod shortcuts;
pub mod tools;
pub mod workspace;

pub use app::StudioSession;
pub use context::{
    ContextRejection, ContextStack, EditContext, EscapeOutcome, HandleId, HandleRef, ItemKind,
    NodeId, SegmentId, SelectionState, SubSelection, VectorOperation,
};
pub use error::{Result, UiError};
pub use input::{PointerEvent, ToolKind, UserAction};
pub use numeric::{InputMethod, NumericField, SizeFields};
pub use shortcuts::{ActionId, BindError, KeyCombo, ShortcutTable};
pub use tools::{
    HitTarget, NodeTool, OverlayPrimitive, PointerSample, SelectTool, ToolController, ToolResponse,
    ToolServices, ToolSession, ViewTransform,
};
pub use workspace::{DockSide, PanelState, ViewState, WorkspaceState};
