//! Data Merge panel controller and presentation interactions (10.11).

use aubrieta_document::{
    BindingId, ChangeSet, DataBinding, DataSourceDefinition, DataSourceId, DataSourceParser,
    PreflightFinding,
};
use aubrieta_foundation::{AubrietaError, SurfaceId};

use crate::bridge::{AubrietaGuiBridge, DataMergePresentationModel};

/// Controller managing the Variable Data / Data Merge panel (10.11).
#[derive(Debug, Default)]
pub struct DataMergePanelController;

impl DataMergePanelController {
    /// Creates a fresh Data Merge panel controller.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Resolves the current presentation model for the Data Merge panel.
    #[must_use]
    pub fn query_model(&self, bridge: &AubrietaGuiBridge) -> DataMergePresentationModel {
        bridge.query_variable_data()
    }

    /// Parses and registers a delimited text data source (CSV/TSV).
    pub fn import_delimited(
        &self,
        bridge: &mut AubrietaGuiBridge,
        id: DataSourceId,
        name: impl Into<String>,
        content: &str,
        delimiter: char,
    ) -> Result<ChangeSet, AubrietaError> {
        let source = DataSourceParser::parse_delimited(id, name, content, delimiter)?;
        bridge.import_data_source(source)
    }

    /// Parses and registers a JSON array data source.
    pub fn import_json(
        &self,
        bridge: &mut AubrietaGuiBridge,
        id: DataSourceId,
        name: impl Into<String>,
        content: &str,
    ) -> Result<ChangeSet, AubrietaError> {
        let source = DataSourceParser::parse_json(id, name, content)?;
        bridge.import_data_source(source)
    }

    /// Registers a pre-parsed data source definition.
    pub fn register_source(
        &self,
        bridge: &mut AubrietaGuiBridge,
        source: DataSourceDefinition,
    ) -> Result<ChangeSet, AubrietaError> {
        bridge.import_data_source(source)
    }

    /// Removes a data source and its cascading bindings.
    pub fn remove_source(
        &self,
        bridge: &mut AubrietaGuiBridge,
        id: DataSourceId,
    ) -> Result<ChangeSet, AubrietaError> {
        bridge.remove_data_source(id)
    }

    /// Adds a data binding between a field and a document object property.
    pub fn add_binding(
        &self,
        bridge: &mut AubrietaGuiBridge,
        binding: DataBinding,
    ) -> Result<ChangeSet, AubrietaError> {
        bridge.add_data_binding(binding)
    }

    /// Removes an active data binding by ID.
    pub fn remove_binding(
        &self,
        bridge: &mut AubrietaGuiBridge,
        id: BindingId,
    ) -> Result<ChangeSet, AubrietaError> {
        bridge.remove_data_binding(id)
    }

    /// Materializes records into generated surfaces on the canvas pasteboard.
    pub fn materialize(
        &self,
        bridge: &mut AubrietaGuiBridge,
        source_id: DataSourceId,
        template_surface: SurfaceId,
    ) -> Result<ChangeSet, AubrietaError> {
        bridge.materialize_merge(source_id, template_surface)
    }

    /// Evaluates preflight findings for a data source against the document template.
    /// Delegates to the shared engine implementation (Table B).
    pub fn preflight(
        &self,
        bridge: &AubrietaGuiBridge,
        source_id: DataSourceId,
    ) -> Result<Vec<PreflightFinding>, AubrietaError> {
        let session = bridge
            .session()
            .ok_or_else(|| AubrietaError::invalid_input("no active document session"))?;
        aubrieta_application::data_merge::preflight(session.document(), source_id)
    }
}
