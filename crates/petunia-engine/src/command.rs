//! Command pattern implementation, undo/redo stack, and document transactions.

use crate::error::{EngineError, Result};
use crate::transaction::{
    apply_ops_atomic, commit_transaction, prepare_transaction, AppliedTransaction, CommandId,
    DocumentOp, DocumentRevision, TransactionRequest,
};
use petunia_core::{Document, ObjectId, ParentRef, SceneNode, Transform2D};

/// Trait implemented by any undoable document modification command.
pub trait Command: std::fmt::Debug + Send + Sync {
    /// Human-readable label for history panels and tooltips.
    fn description(&self) -> &str;

    /// Applies the command to the document.
    fn execute(&mut self, document: &mut Document) -> Result<()>;

    /// Reverts the command changes in the document.
    fn undo(&mut self, document: &mut Document) -> Result<()>;
}

/// Command to add an object to the scene graph.
#[derive(Debug)]
pub struct AddNodeCommand {
    node: Option<SceneNode>,
    applied: Option<AppliedTransaction>,
}

impl AddNodeCommand {
    #[must_use]
    pub fn new(node: SceneNode) -> Self {
        Self {
            node: Some(node),
            applied: None,
        }
    }
}

impl Command for AddNodeCommand {
    fn description(&self) -> &str {
        "Add Node"
    }

    fn execute(&mut self, document: &mut Document) -> Result<()> {
        if self.applied.is_some() {
            return Err(EngineError::Execution("command already executed".into()));
        }
        let node = self
            .node
            .as_ref()
            .ok_or_else(|| EngineError::Execution("command node missing".into()))?
            .clone();
        let operation = match node.parent {
            ParentRef::Page(page) => DocumentOp::InsertRoot {
                index: document
                    .scene
                    .page_roots(page)
                    .ok_or_else(|| EngineError::Execution("page missing".into()))?
                    .len(),
                node: Box::new(node),
            },
            ParentRef::Object(parent) => DocumentOp::InsertNode {
                parent,
                index: document
                    .scene
                    .children_of(parent)
                    .ok_or_else(|| EngineError::Execution("group missing".into()))?
                    .len(),
                node: Box::new(node),
            },
        };
        let prepared = prepare_transaction(
            document,
            TransactionRequest {
                command_id: CommandId::new_v4(),
                operations: vec![operation],
                merge_key: None,
            },
            DocumentRevision::GENESIS,
        )
        .map_err(|error| EngineError::Execution(error.to_string()))?;
        let applied = commit_transaction(document, prepared)
            .map_err(|error| EngineError::Execution(error.to_string()))?;
        self.applied = Some(applied);
        Ok(())
    }

    fn undo(&mut self, document: &mut Document) -> Result<()> {
        let applied = self
            .applied
            .as_ref()
            .ok_or_else(|| EngineError::Execution("nothing to undo".into()))?;
        apply_ops_atomic(document, &applied.inverse)?;
        self.applied = None;
        Ok(())
    }
}

/// Command to apply an affine transform to an existing node.
#[derive(Debug)]
pub struct TransformNodeCommand {
    id: ObjectId,
    new_transform: Transform2D,
    previous_transform: Option<Transform2D>,
}

impl TransformNodeCommand {
    #[must_use]
    pub const fn new(id: ObjectId, transform: Transform2D) -> Self {
        Self {
            id,
            new_transform: transform,
            previous_transform: None,
        }
    }
}

impl Command for TransformNodeCommand {
    fn description(&self) -> &str {
        "Transform Node"
    }

    fn execute(&mut self, document: &mut Document) -> Result<()> {
        let node = document.scene.get_node(self.id).ok_or_else(|| {
            EngineError::Core(petunia_core::CoreError::ObjectNotFound(self.id.to_string()))
        })?;
        let previous = node.transform;
        apply_ops_atomic(
            document,
            &[DocumentOp::SetTransform {
                object: self.id,
                transform: self.new_transform,
            }],
        )?;
        self.previous_transform = Some(previous);
        Ok(())
    }

    fn undo(&mut self, document: &mut Document) -> Result<()> {
        let previous = self
            .previous_transform
            .ok_or_else(|| EngineError::Execution("no previous transform saved".into()))?;
        apply_ops_atomic(
            document,
            &[DocumentOp::SetTransform {
                object: self.id,
                transform: previous,
            }],
        )?;
        self.previous_transform = None;
        Ok(())
    }
}

/// History stack managing undo and redo buffers.
#[derive(Debug, Default)]
pub struct CommandHistory {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
}

impl CommandHistory {
    #[must_use]
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Executes a command on the document and records it on the undo stack.
    pub fn execute(
        &mut self,
        mut command: Box<dyn Command>,
        document: &mut Document,
    ) -> Result<()> {
        let mut staged = document.clone();
        command.execute(&mut staged)?;
        staged.validate()?;
        *document = staged;
        self.undo_stack.push(command);
        self.redo_stack.clear();
        Ok(())
    }

    /// Undoes the last command.
    pub fn undo(&mut self, document: &mut Document) -> Result<()> {
        let mut command = self.undo_stack.pop().ok_or(EngineError::NothingToUndo)?;
        let mut staged = document.clone();
        if let Err(error) = command
            .undo(&mut staged)
            .and_then(|()| staged.validate().map_err(EngineError::from))
        {
            self.undo_stack.push(command);
            return Err(error);
        }
        *document = staged;
        self.redo_stack.push(command);
        Ok(())
    }

    /// Redoes the most recently undone command.
    pub fn redo(&mut self, document: &mut Document) -> Result<()> {
        let mut command = self.redo_stack.pop().ok_or(EngineError::NothingToRedo)?;
        let mut staged = document.clone();
        if let Err(error) = command
            .execute(&mut staged)
            .and_then(|()| staged.validate().map_err(EngineError::from))
        {
            self.redo_stack.push(command);
            return Err(error);
        }
        *document = staged;
        self.undo_stack.push(command);
        Ok(())
    }

    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petunia_core::VectorPath;

    #[test]
    fn test_command_undo_redo_cycle() {
        let mut doc = Document::new("Undo Test");
        let mut history = CommandHistory::new();

        let page = doc.scene.default_page();
        let node = SceneNode::new_path(
            "Box",
            VectorPath::rect(0.0, 0.0, 100.0, 100.0),
            petunia_core::ParentRef::Page(page),
        );
        let id = node.id;

        history
            .execute(Box::new(AddNodeCommand::new(node)), &mut doc)
            .expect("execute succeeds");
        assert_eq!(doc.scene.len(), 1);
        assert!(history.can_undo());
        assert!(!history.can_redo());

        // Undo
        history.undo(&mut doc).expect("undo succeeds");
        assert_eq!(doc.scene.len(), 0);
        assert!(!history.can_undo());
        assert!(history.can_redo());

        // Redo
        history.redo(&mut doc).expect("redo succeeds");
        assert_eq!(doc.scene.len(), 1);
        assert!(doc.scene.get_node(id).is_some());
    }
}
