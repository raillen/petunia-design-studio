//! Immutable presentation models and view-models for UI consumption (09.27).
//!
//! Rules:
//! - Identified by stable semantic IDs;
//! - Cheap to clone or diff;
//! - No hidden mutation methods;
//! - No toolkit types or raw widget dependencies;
//! - Localization keys and tokens rather than hardcoded UI strings.

use std::collections::HashMap;

use petunia_design_document::ContainerRole;
use petunia_design_foundation::{ObjectId, SurfaceId};
use serde::{Deserialize, Serialize};

use crate::ActionId;

/// High-level session snapshot for title bar, tabs, and shell status.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionSnapshot {
    /// Active surface ID, if any.
    pub active_surface: Option<SurfaceId>,
    /// Document display title.
    pub title: String,
    /// Monotonic revision number.
    pub revision: u64,
    /// Whether document has unsaved modifications.
    pub is_dirty: bool,
    /// Number of surfaces in the document.
    pub surface_count: usize,
    /// Total count of objects across surfaces.
    pub total_objects: usize,
    /// Number of currently selected objects.
    pub selected_count: usize,
    /// Whether undo is currently available.
    pub can_undo: bool,
    /// Whether redo is currently available.
    pub can_redo: bool,
}

/// Document-level summary metrics.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentSummary {
    /// Document title.
    pub title: String,
    /// Number of surfaces.
    pub surface_count: usize,
    /// Total objects across all surfaces.
    pub total_objects: usize,
    /// Monotonic revision.
    pub revision: u64,
    /// Unsaved modifications present.
    pub is_dirty: bool,
}

/// Selection summary without widget ownership.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SelectionViewModel {
    /// Stable IDs of selected objects in intentional order.
    pub selected_ids: Vec<ObjectId>,
    /// Key (primary) object for alignment and property anchoring.
    pub key_object: Option<ObjectId>,
    /// Combined bounding box `[x, y, width, height]` in document points.
    pub combined_bounds: Option<[f64; 4]>,
    /// Number of selected objects.
    pub count: usize,
    /// True when nothing is selected.
    pub is_empty: bool,
}

impl SelectionViewModel {
    /// Checks if a specific object ID is selected.
    #[must_use]
    pub fn contains(&self, id: ObjectId) -> bool {
        self.selected_ids.contains(&id)
    }
}

/// Single row in the unified layers tree presentation model (10.5).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayerRowViewModel {
    /// Stable object ID.
    pub id: ObjectId,
    /// Surface where this object resides.
    pub surface_id: SurfaceId,
    /// Object human-readable name.
    pub name: String,
    /// Visibility status.
    pub visible: bool,
    /// Locked status against interactive edits.
    pub locked: bool,
    /// Whether this row is currently selected.
    pub is_selected: bool,
    /// Hierarchy nesting depth (0 = top-level child of surface).
    pub depth: usize,
    /// Parent container object ID, if any.
    pub parent_id: Option<ObjectId>,
    /// Whether this object is a container (group, layer, clip group).
    pub is_container: bool,
    /// Container role if this object acts as a container.
    pub role: Option<ContainerRole>,
    /// Whether this object is a clipping mask boundary.
    pub is_clip_mask: bool,
    /// Target clipping mask object ID, if clipped.
    pub clip_mask_id: Option<ObjectId>,
    /// Number of direct children, if container.
    pub children_count: usize,
    /// Semantic fill token, if any.
    pub fill_token: Option<String>,
    /// Semantic stroke token, if any.
    pub stroke_token: Option<String>,
    /// Opacity factor in `[0.0, 1.0]`.
    pub opacity: f64,
    /// Evaluated bounding box `[x, y, w, h]`, if defined.
    pub bounds: Option<[f64; 4]>,
}

/// Presentation model for a surface container in the layers panel.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SurfaceRowViewModel {
    /// Stable surface ID.
    pub id: SurfaceId,
    /// Surface human-readable name.
    pub name: String,
    /// True if this is the currently active/focused surface.
    pub is_active: bool,
    /// Number of objects in this surface.
    pub object_count: usize,
    /// Pasteboard origin `[x, y]` in document units.
    pub origin: [f64; 2],
    /// Surface dimensions `[width, height]`.
    pub dimensions: [f64; 2],
    /// Bleed margins.
    pub bleed: petunia_design_document::Bleed,
    /// Safe margin insets.
    pub margins: petunia_design_document::Margins,
    /// Background color token or hex.
    pub background: Option<String>,
    /// Number of guides defined on this surface.
    pub guide_count: usize,
}

/// Complete presentation model for the Layers Panel (10.5).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LayersPresentationModel {
    /// Available surfaces.
    pub surfaces: Vec<SurfaceRowViewModel>,
    /// Flattened display rows representing the canonical tree.
    pub rows: Vec<LayerRowViewModel>,
    /// Total count of rows.
    pub total_count: usize,
    /// Number of selected rows.
    pub selected_count: usize,
}

/// Presentation model for the Properties Inspector (09.25, 10.1, 10.4, 10.7).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PropertiesPresentationModel {
    /// Whether the selection is empty (controls inspect canvas/document properties).
    pub selection_empty: bool,
    /// Whether multiple objects with heterogeneous values are selected.
    pub is_mixed: bool,
    /// Name of the primary selected object.
    pub name: Option<String>,
    /// Fill token reference or hex string.
    pub fill: Option<String>,
    /// Stroke token reference.
    pub stroke: Option<String>,
    /// Stroke line width.
    pub stroke_width: f64,
    /// Opacity factor in `[0.0, 1.0]`.
    pub opacity: f64,
    /// Visibility status.
    pub visible: bool,
    /// Locked status.
    pub locked: bool,
    /// Coordinate and dimension bounds `[x, y, width, height]`.
    pub bounds: Option<[f64; 4]>,
    /// Rotation angle in radians.
    pub rotation: f64,
    /// Canonical V1 appearance stack if defined on primary object.
    pub appearance: Option<petunia_design_document::AppearanceStack>,
    /// Active surface layout metadata when selection is empty (10.7).
    pub active_surface: Option<SurfaceRowViewModel>,
}

impl Default for PropertiesPresentationModel {
    fn default() -> Self {
        Self {
            selection_empty: true,
            is_mixed: false,
            name: None,
            fill: None,
            stroke: None,
            stroke_width: 1.0,
            opacity: 1.0,
            visible: true,
            locked: false,
            bounds: None,
            rotation: 0.0,
            appearance: None,
            active_surface: None,
        }
    }
}

/// History entry presentation descriptor.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HistoryItemViewModel {
    /// Stable zero-based index in history list.
    pub index: usize,
    /// Human-readable description of the operation.
    pub description: String,
    /// Number of changes produced by this command.
    pub change_count: usize,
}

/// Presentation model for the History Panel / Undo Stack (09.3).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HistoryPresentationModel {
    /// Undoable actions in order from oldest to newest.
    pub undo_stack: Vec<HistoryItemViewModel>,
    /// Redoable actions in reverse order.
    pub redo_stack: Vec<HistoryItemViewModel>,
    /// Whether undo is currently available.
    pub can_undo: bool,
    /// Whether redo is currently available.
    pub can_redo: bool,
    /// Current undo step label, if any.
    pub active_undo_label: Option<String>,
}

/// Action state for command menus, buttons, and shortcuts (09.27).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActionStateViewModel {
    /// Action identifier.
    pub action_id: ActionId,
    /// Whether the action is currently invokable.
    pub is_enabled: bool,
    /// Whether the action is in a toggled/checked state.
    pub is_checked: bool,
    /// Diagnostic or help reason if disabled.
    pub disabled_reason: Option<String>,
}

/// Map of all action states by ID.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ActionStateMap {
    /// Inner state mapping.
    pub states: HashMap<String, ActionStateViewModel>,
}

impl ActionStateMap {
    /// Looks up the state for an action.
    ///
    /// Read-side migration only (15.A): the map is keyed by canonical
    /// `ptnd.action.*` ids, but a caller still holding a pre-grammar spelling
    /// (`ptnd.file.save`) or a legacy `aubrieta.*` id resolves too. The map
    /// never *emits* the old spelling.
    #[must_use]
    pub fn get(&self, action: &ActionId) -> Option<&ActionStateViewModel> {
        if let Some(state) = self.states.get(action.as_str()) {
            return Some(state);
        }
        let canonical = petunia_design_foundation::normalize_action_id(
            &petunia_design_foundation::normalized(action.as_str()),
        );
        self.states.get(&canonical)
    }
}

#[cfg(test)]
mod action_state_map_tests {
    use super::*;

    fn map_with(id: &str, enabled: bool) -> ActionStateMap {
        let mut states = HashMap::new();
        states.insert(
            id.to_string(),
            ActionStateViewModel {
                action_id: ActionId::new(id),
                is_enabled: enabled,
                is_checked: false,
                disabled_reason: None,
            },
        );
        ActionStateMap { states }
    }

    #[test]
    fn lookup_accepts_canonical_spellings() {
        let map = map_with("ptnd.action.file.save", true);
        assert!(map
            .get(&ActionId::new("ptnd.action.file.save"))
            .is_some_and(|state| state.is_enabled));
    }

    #[test]
    fn lookup_migrates_read_side_ids_without_emitting_them() {
        let map = map_with("ptnd.action.file.save", true);
        assert!(map.get(&ActionId::new("ptnd.file.save")).is_some());
        assert!(map
            .get(&ActionId::new("aubrieta.action.file.save"))
            .is_some());
        assert!(map.get(&ActionId::new("ptnd.action.file.nope")).is_none());
    }
}

/// Semantic dialog requests emitted from core to UI adapters (09.27).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DialogRequest {
    /// Request to select a file for opening or importing.
    ChooseOpenFile {
        title: String,
        extensions: Vec<String>,
    },
    /// Request to choose a file path for saving or exporting.
    ChooseSaveDestination {
        default_name: String,
        extension: String,
    },
    /// Request confirmation for a destructive action.
    ConfirmDestructiveAction {
        title: String,
        message: String,
        confirm_label: String,
    },
}

/// Field metadata view-model for data merge UI (10.11).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldViewModel {
    /// Stable field ID.
    pub id: petunia_design_document::FieldId,
    /// Field display title.
    pub name: String,
    /// Semantic data type.
    pub field_type: petunia_design_document::FieldType,
}

/// Data source summary view-model for data merge panel (10.11).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataSourceViewModel {
    /// Stable data source ID.
    pub id: petunia_design_document::DataSourceId,
    /// Display name.
    pub name: String,
    /// Underlying tabular format.
    pub format: petunia_design_document::DataSourceFormat,
    /// Number of available fields.
    pub field_count: usize,
    /// Number of records/rows.
    pub record_count: usize,
    /// Field descriptors.
    pub fields: Vec<FieldViewModel>,
}

/// Active data binding descriptor for panel presentation (10.11).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataBindingViewModel {
    /// Stable binding ID.
    pub id: petunia_design_document::BindingId,
    /// Data source ID.
    pub source_id: petunia_design_document::DataSourceId,
    /// Field ID.
    pub field_id: petunia_design_document::FieldId,
    /// Bound field name.
    pub field_name: String,
    /// Target object ID.
    pub target_object: ObjectId,
    /// Target object name.
    pub target_object_name: String,
    /// Target property path.
    pub target_property: petunia_design_document::TargetProperty,
    /// Declarative formatter.
    pub formatter: petunia_design_document::ValueFormatter,
}

/// Complete presentation model for the Variable Data / Data Merge panel (10.11).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DataMergePresentationModel {
    /// Registered data sources.
    pub sources: Vec<DataSourceViewModel>,
    /// Active property bindings.
    pub bindings: Vec<DataBindingViewModel>,
    /// Currently previewed record index (0-based), if any.
    pub preview_record: Option<usize>,
    /// Total records across primary source.
    pub total_records: usize,
}
