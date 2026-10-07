//! Command pattern implementation, undo/redo stack, and document transactions.

use crate::error::{EngineError, Result};
use petunia_core::{Document, ObjectId, SceneNode, Transform2D};

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
    added_id: Option<ObjectId>,
}

impl AddNodeCommand {
    #[must_use]
    pub fn new(node: SceneNode) -> Self {
        Self {
            node: Some(node),
            added_id: None,
        }
    }
}

impl Command for AddNodeCommand {
    fn description(&self) -> &str {
        "Add Node"
    }

    fn execute(&mut self, document: &mut Document) -> Result<()> {
        let node = self
            .node
            .take()
            .ok_or_else(|| EngineError::Execution("Command already executed".into()))?;
        let id = node.id;
        document.scene.insert_node(node);
        self.added_id = Some(id);
        Ok(())
    }

    fn undo(&mut self, document: &mut Document) -> Result<()> {
        let id = self
            .added_id
            .take()
            .ok_or_else(|| EngineError::Execution("Nothing to undo".into()))?;
        let removed = document.scene.remove_node(id)?;
        self.node = Some(removed);
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
        let node = document
            .scene
            .get_node_mut(self.id)
            .ok_or_else(|| EngineError::Core(petunia_core::CoreError::ObjectNotFound(self.id.to_string())))?;
        self.previous_transform = Some(node.transform);
        node.transform = self.new_transform;
        Ok(())
    }

    fn undo(&mut self, document: &mut Document) -> Result<()> {
        let prev = self
            .previous_transform
            .take()
            .ok_or_else(|| EngineError::Execution("No previous transform saved".into()))?;
        let node = document
            .scene
            .get_node_mut(self.id)
            .ok_or_else(|| EngineError::Core(petunia_core::CoreError::ObjectNotFound(self.id.to_string())))?;
        node.transform = prev;
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
    pub fn execute(&mut self, mut command: Box<dyn Command>, document: &mut Document) -> Result<()> {
        command.execute(document)?;
        self.undo_stack.push(command);
        self.redo_stack.clear();
        Ok(())
    }

    /// Undoes the last command.
    pub fn undo(&mut self, document: &mut Document) -> Result<()> {
        let mut command = self.undo_stack.pop().ok_or(EngineError::NothingToUndo)?;
        command.undo(document)?;
        self.redo_stack.push(command);
        Ok(())
    }

    /// Redoes the most recently undone command.
    pub fn redo(&mut self, document: &mut Document) -> Result<()> {
        let mut command = self.redo_stack.pop().ok_or(EngineError::NothingToRedo)?;
        command.execute(document)?;
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

        let node = SceneNode::new_path("Box", VectorPath::rect(0.0, 0.0, 100.0, 100.0));
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
