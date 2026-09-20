//! Undoable commands executed through [`aubrieta_document::DocumentMutator`].

use aubrieta_document::{ChangeSet, Document, DocumentMutator, DocumentObject};
use aubrieta_foundation::{AubrietaError, ObjectId, SurfaceId};

/// Single undoable command with explicit IDs (no hidden state).
#[derive(Clone, Debug)]
pub enum Command {
    /// Create a surface.
    CreateSurface { id: SurfaceId, name: String },
    /// Create an object on a surface.
    CreateObject {
        surface: SurfaceId,
        id: ObjectId,
        name: String,
    },
    /// Delete an object by stable ID.
    DeleteObject { id: ObjectId },
    /// Set an object's semantic fill token.
    SetFill { id: ObjectId, fill: Option<String> },
    /// Set an object's visibility flag.
    SetVisibility { id: ObjectId, visible: bool },
    /// Set an object's locked flag.
    SetLocked { id: ObjectId, locked: bool },
    /// Set an object's opacity factor in [0.0, 1.0].
    SetOpacity { id: ObjectId, opacity: f64 },
    /// Set an object's stroke and stroke width.
    SetStroke {
        id: ObjectId,
        stroke: Option<String>,
        width: f64,
    },
    /// Set an object's bounds and rotation.
    SetBounds {
        id: ObjectId,
        bounds: Option<[f64; 4]>,
        rotation: f64,
    },
    /// Reorder an object within a surface.
    ReorderObject {
        surface: SurfaceId,
        id: ObjectId,
        new_index: usize,
    },
    /// Set an object's complete appearance stack (10.4).
    SetAppearance {
        id: ObjectId,
        appearance: Option<aubrieta_document::AppearanceStack>,
    },
}

/// Validated command ready for execution.
#[derive(Clone, Debug)]
pub struct CommandRequest {
    /// Command to execute.
    pub command: Command,
}

impl CommandRequest {
    /// Wraps a command.
    #[must_use]
    pub fn new(command: Command) -> Self {
        Self { command }
    }
}

/// Executes one command, returning the produced [`ChangeSet`].
pub fn execute(
    document: &mut Document,
    request: &CommandRequest,
) -> Result<ChangeSet, AubrietaError> {
    let mut mutator = DocumentMutator::new(document);
    match &request.command {
        Command::CreateSurface { id, name } => mutator.add_surface(*id, name.clone()),
        Command::CreateObject { surface, id, name } => {
            mutator.add_object(*surface, DocumentObject::new(*id, name.clone()))
        }
        Command::DeleteObject { id } => mutator.remove_object(*id),
        Command::SetFill { id, fill } => mutator.set_fill(*id, fill.clone()),
        Command::SetVisibility { id, visible } => mutator.set_visibility(*id, *visible),
        Command::SetLocked { id, locked } => mutator.set_locked(*id, *locked),
        Command::SetOpacity { id, opacity } => mutator.set_opacity(*id, *opacity),
        Command::SetStroke { id, stroke, width } => mutator.set_stroke(*id, stroke.clone(), *width),
        Command::SetBounds {
            id,
            bounds,
            rotation,
        } => mutator.set_bounds(*id, *bounds, *rotation),
        Command::ReorderObject {
            surface,
            id,
            new_index,
        } => mutator.reorder_object(*surface, *id, *new_index),
        Command::SetAppearance { id, appearance } => {
            mutator.set_appearance(*id, appearance.clone())
        }
    }
}
