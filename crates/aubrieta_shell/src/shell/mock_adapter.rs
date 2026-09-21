//! Mock GUI adapter for CI and headless conformance verification (09.27).
//!
//! Validates the 8 mandatory conformance requirements of 09.27 lines 141-153:
//! 1. Enumerate semantic actions;
//! 2. Resolve state snapshots;
//! 3. Submit property edits / commands;
//! 4. Observe jobs / history;
//! 5. Handle dialog requests symbolically;
//! 6. Receive presentation deltas;
//! 7. Open / close document sessions;
//! 8. Run without visual assets resolved.

use aubrieta_application::CommandRequest;
use aubrieta_document::{ChangeSet, Document};
use aubrieta_foundation::{AubrietaError, ObjectId};

use crate::bridge::{
    ActionStateMap, AubrietaGuiBridge, DialogRequest, HistoryPresentationModel,
    LayersPresentationModel, PropertiesPresentationModel, SessionSnapshot,
};

/// Toolkit-free mock GUI adapter exercising canonical application contracts headlessly.
#[derive(Debug, Default)]
pub struct MockGuiAdapter {
    /// Inner application facade.
    pub bridge: AubrietaGuiBridge,
    /// Recorded dialog requests handled symbolically.
    pub handled_dialogs: Vec<DialogRequest>,
    /// Accumulated change sets received across mutations.
    pub change_log: Vec<ChangeSet>,
}

impl MockGuiAdapter {
    /// Creates a fresh mock adapter.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 1. Enumerate semantic actions and their active states.
    #[must_use]
    pub fn enumerate_actions(&self) -> ActionStateMap {
        self.bridge.query_actions()
    }

    /// 2. Resolve state snapshots.
    #[must_use]
    pub fn resolve_snapshot(&self) -> SessionSnapshot {
        self.bridge.snapshot()
    }

    /// 3. Submit commands and property edits.
    pub fn submit_command(&mut self, req: CommandRequest) -> Result<ChangeSet, AubrietaError> {
        let changes = self.bridge.submit_command(req)?;
        self.change_log.push(changes.clone());
        Ok(changes)
    }

    /// Submits property edit: fill.
    pub fn edit_fill(
        &mut self,
        id: ObjectId,
        fill: Option<String>,
    ) -> Result<ChangeSet, AubrietaError> {
        let changes = self.bridge.set_fill(id, fill)?;
        self.change_log.push(changes.clone());
        Ok(changes)
    }

    /// 4. Observe history and undo/redo stacks.
    #[must_use]
    pub fn observe_history(&self) -> HistoryPresentationModel {
        self.bridge.query_history()
    }

    /// 5. Handle dialog requests symbolically without native OS dialogs.
    pub fn handle_dialog(&mut self, dialog: DialogRequest) {
        self.handled_dialogs.push(dialog);
    }

    /// 6. Receive presentation deltas (layers and properties models).
    #[must_use]
    pub fn observe_presentation_deltas(
        &self,
    ) -> (LayersPresentationModel, PropertiesPresentationModel) {
        (self.bridge.query_layers(), self.bridge.query_properties())
    }

    /// 7. Open and close document sessions.
    pub fn open_session(
        &mut self,
        title: impl Into<String>,
        document: Document,
    ) -> Result<(), AubrietaError> {
        self.bridge.open_document(title, document)
    }

    /// Closes the active session.
    pub fn close_session(&mut self, force: bool) -> Result<bool, AubrietaError> {
        self.bridge.close_session(force)
    }

    /// 8. Verifies that the adapter executes end-to-end headlessly without graphical assets.
    #[must_use]
    pub fn is_headless_compliant(&self) -> bool {
        // True if full state snapshot can be queried without visual asset dependencies
        let snap = self.resolve_snapshot();
        let _actions = self.enumerate_actions();
        let _history = self.observe_history();
        let (layers, _props) = self.observe_presentation_deltas();
        snap.surface_count == layers.surfaces.len()
    }
}
