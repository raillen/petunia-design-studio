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
    }
}
