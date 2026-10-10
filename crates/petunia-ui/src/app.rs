//! Application session: the document writer and tool dispatch.
//!
//! The session is the single writer. Every mutation travels as a
//! transaction; hover never mutates, and one confirmed gesture
//! commits at most one Transaction.

use crate::context::{
    ContextStack, EditContext, EscapeOutcome, HandleId, HandleRef, NodeId, SelectionState,
    VectorOperation,
};
use crate::error::Result;
use crate::focus::{FocusEntry, FocusManager, FocusOutcome};
use crate::input::{PointerEvent, ToolKind, UserAction};
use crate::layers::{LayerKey, LayerKind, LayerRow, LayersPanel};
use crate::numeric::NumericField;
use crate::shortcuts::ActionId;
use crate::shortcuts::ShortcutTable;
use crate::tools::{
    distance, HitTarget, ItemKind, NodeTool, PenTool, PointerSample, SelectTool, SelectionDelta,
    ToolController, ToolResponse, ToolServices, ToolSession, ViewTransform,
};
use crate::tooltips::tooltip_for;
use crate::workspace::{ViewState, WorkspaceState};
use petunia_core::{Document, ObjectId, ParentRef, Point, SceneItem, SceneNode, Size2, VectorPath};
use petunia_engine::compile;
use petunia_engine::geometry::{
    delete_node, flatten_contour, preview_delete, SmartDeleteMode, SmartDeleteOutcome,
};

use petunia_engine::ptnd::{BlobStore, FileFingerprint};
use petunia_engine::spatial::{SnapLatch, SnapSettings};
use petunia_engine::EngineError;
use petunia_engine::History;
use petunia_engine::{prepare_transaction, TransactionRequest};
use petunia_engine::{CommandId, DocumentOp, DocumentRevision, HistoryDescription};
use petunia_render::RenderOptions;
use petunia_render::SoftwareRenderer;
use petunia_render_model::RenderQuality;
use petunia_render_model::RenderStats;
use std::cell::Cell;
mod runtime;
pub use runtime::NativeClipboard;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::Arc;

/// Ghost preview of a Smart Delete: candidate curve points plus the
/// honest error number the UI displays next to them.
#[derive(Debug, Clone)]
pub struct SmartDeleteGhost {
    /// Document-space points of the candidate curve.
    pub points: Vec<Point>,
    /// Largest deviation between source and candidate geometry.
    pub max_error: f64,
    /// Whether the candidate fits the declared tolerance.
    pub within_tolerance: bool,
}

/// Studio session. Owns the document, its history, and every piece
/// of Session State (contexts, selection, view, workspace).
pub struct StudioSession {
    document: Document,
    history: History,
    active_tool: ToolKind,
    contexts: ContextStack,
    selection: SelectionState,
    view: ViewState,
    workspace: WorkspaceState,
    shortcuts: ShortcutTable,
    x_field: NumericField,
    y_field: NumericField,
    focus: FocusManager,
    tools: SessionTools,
    layers: LayersPanel,
    layers_revision: DocumentRevision,
    layers_selection: Vec<ObjectId>,
    show_multi_node_bounds: bool,
    captured: bool,
    active_page: petunia_core::PageId,
    blobs: BlobStore,
    fonts: petunia_engine::text::FontRegistry,
    images: HashMap<petunia_core::ResourceId, Arc<petunia_render_model::image::ResolvedImage>>,
    extensions: BTreeMap<String, Vec<u8>>,
    preview: Option<Vec<u8>>,
    destination: Option<(PathBuf, FileFingerprint)>,
    snap_latch: Cell<Option<SnapLatch>>,
    snap_settings: SnapSettings,
}

/// One controller of each kind, owned by the session. Pointer events
/// clone the active one out, run it, and store it back, so gesture
/// state survives from Down to Up with no aliased borrows.
#[derive(Debug, Clone, Default)]
struct SessionTools {
    select: SelectTool,
    node: NodeTool,
    pen: PenTool,
}

impl StudioSession {
    /// A fresh session with an empty document.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        let document = Document::new(name);
        let active_page = document.scene.default_page();
        Self {
            document,
            history: History::new(64 << 20, 256 << 20),
            active_tool: ToolKind::Select,
            contexts: ContextStack::new(),
            selection: SelectionState::new(),
            view: ViewState::default(),
            workspace: WorkspaceState::defaults(),
            shortcuts: ShortcutTable::defaults(),
            x_field: NumericField::new(0.0, 1.0),
            y_field: NumericField::new(0.0, 1.0),
            focus: FocusManager::new(),
            tools: SessionTools::default(),
            layers: LayersPanel::new(),
            layers_revision: DocumentRevision(0),
            layers_selection: Vec::new(),
            show_multi_node_bounds: true,
            captured: false,
            active_page,
            blobs: BlobStore::new(),
            fonts: petunia_engine::text::FontRegistry::new(),
            images: HashMap::new(),
            extensions: BTreeMap::new(),
            preview: None,
            destination: None,
            snap_latch: Cell::new(None),
            snap_settings: SnapSettings::default(),
        }
    }

    /// Current authorial revision.
    #[must_use]
    pub fn revision(&self) -> DocumentRevision {
        self.history.current_revision()
    }

    /// Whether one committed transaction can be undone.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Whether one transaction can be redone.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Project retained history without exposing the writer to panels.
    #[must_use]
    pub fn history_rows(&self) -> Vec<(HistoryDescription, DocumentRevision, bool)> {
        let (entries, applied) = self.history.entries();
        entries
            .iter()
            .enumerate()
            .map(|(index, entry)| (entry.description, entry.after_revision, index < applied))
            .collect()
    }

    /// Pending input is Session State and must be resolved before switching.
    #[must_use]
    pub fn interaction_active(&self) -> bool {
        self.captured || !self.tools.pen.points().is_empty()
    }

    /// Select known editable objects from a semantic GUI panel.
    pub fn set_selection(&mut self, objects: Vec<ObjectId>) -> Result<()> {
        if objects.iter().any(|id| !self.is_editable(*id)) {
            return Err(crate::UiError::State(
                "o objeto está ausente, oculto ou bloqueado".into(),
            ));
        }
        let mut unique = Vec::with_capacity(objects.len());
        for id in objects {
            if !unique.contains(&id) {
                unique.push(id);
            }
        }
        self.selection.set_objects(unique);
        self.sync_layers();
        Ok(())
    }

    /// Read-only document access.
    #[must_use]
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Current context stack.
    #[must_use]
    pub fn contexts(&self) -> &ContextStack {
        &self.contexts
    }

    /// Current selection (Session State).
    #[must_use]
    pub fn selection(&self) -> &SelectionState {
        &self.selection
    }

    /// Current view state (Session State).
    #[must_use]
    pub fn view(&self) -> &ViewState {
        &self.view
    }

    /// Mutable view state (zoom, pan, DPR).
    pub fn view_mut(&mut self) -> &mut ViewState {
        &mut self.view
    }

    /// Current workspace arrangement (Session State).
    #[must_use]
    pub fn workspace(&self) -> &WorkspaceState {
        &self.workspace
    }

    /// Mutable workspace arrangement.
    pub fn workspace_mut(&mut self) -> &mut WorkspaceState {
        &mut self.workspace
    }

    /// Shortcut table.
    #[must_use]
    pub fn shortcuts(&self) -> &ShortcutTable {
        &self.shortcuts
    }

    /// Mutable shortcut table for the editor UI.
    pub fn shortcuts_mut(&mut self) -> &mut ShortcutTable {
        &mut self.shortcuts
    }

    /// X coordinate field for the transform inspector.
    #[must_use]
    pub fn x_field(&self) -> &NumericField {
        &self.x_field
    }

    /// Mutable X coordinate field.
    pub fn x_field_mut(&mut self) -> &mut NumericField {
        &mut self.x_field
    }

    /// Keyboard focus manager for the whole window.
    #[must_use]
    pub fn focus(&self) -> &FocusManager {
        &self.focus
    }

    /// Mutable focus manager, for the GUI to feed layout changes.
    pub fn focus_mut(&mut self) -> &mut FocusManager {
        &mut self.focus
    }

    /// Replace the window focusable set, following the layout.
    pub fn set_focus_entries(&mut self, entries: Vec<FocusEntry>) {
        self.focus.set_entries(entries);
    }

    /// Tooltip of an action, reading the live shortcut table.
    #[must_use]
    pub fn tooltip(&self, action: ActionId) -> Option<String> {
        tooltip_for(action, &self.shortcuts).map(|tooltip| tooltip.display())
    }

    /// Escape routed through the focus model first (D9): a dialog
    /// closes, another region returns to the canvas, and only then
    /// the context stack unwinds.
    pub fn on_escape_focus(&mut self) -> (FocusOutcome, EscapeOutcome) {
        let focus = self.focus.on_escape();
        let context = if matches!(focus, FocusOutcome::Unhandled) {
            self.on_escape()
        } else {
            // The focus layer consumed the press; the context stack
            // stays exactly where it was.
            EscapeOutcome::AtRoot
        };
        (focus, context)
    }

    /// Mutable Y coordinate field.
    pub fn y_field_mut(&mut self) -> &mut NumericField {
        &mut self.y_field
    }

    /// The selection's size, for the inspector.
    #[must_use]
    pub fn selection_size(&self) -> Size2 {
        Size2::new(self.x_field.value(), self.y_field.value())
            .unwrap_or(Size2::new(0.0, 0.0).expect("usable"))
    }

    /// Active tool kind.
    #[must_use]
    pub fn active_tool(&self) -> ToolKind {
        self.active_tool
    }

    /// Select a tool; switching never mutates the document.
    /// A switch also drops any in-flight gesture state.
    pub fn select_tool(&mut self, tool: ToolKind) {
        self.active_tool = tool;
        self.tools = SessionTools::default();
    }

    /// Press Escape, following the approved ordering.
    pub fn on_escape(&mut self) -> EscapeOutcome {
        if self.interaction_active() {
            match self.active_tool {
                ToolKind::NodeEdit => {
                    let mut tool = self.tools.node.clone();
                    let outcome = tool.cancel(self);
                    self.tools.node = tool;
                    let _ = outcome;
                }
                ToolKind::Pen => {
                    let mut tool = self.tools.pen.clone();
                    let outcome = tool.cancel(self);
                    self.tools.pen = tool;
                    let _ = outcome;
                }
                _ => {
                    let mut tool = self.tools.select.clone();
                    let outcome = tool.cancel(self);
                    self.tools.select = tool;
                    let _ = outcome;
                }
            };
            self.captured = false;
            return EscapeOutcome::ReturnedToNode;
        }
        self.contexts.on_escape()
    }

    /// Enter: open Vector Edit on an eligible single path.
    pub fn on_enter(&mut self) {
        if self.contexts.in_vector() {
            return;
        }
        if let Some(id) = self.selection.single() {
            if self.is_editable(id) {
                self.contexts.push(EditContext::Vector {
                    targets: vec![id],
                    operation: VectorOperation::Node,
                });
            }
        }
    }

    /// Whether discrete bounding boxes appear automatically for 2+
    /// sub-selected nodes (Decision C of A–F). Session State only.
    #[must_use]
    pub fn show_multi_node_bounds(&self) -> bool {
        self.show_multi_node_bounds
    }

    /// Toggle or set discrete multi-node bounding box visibility.
    pub fn set_show_multi_node_bounds(&mut self, show: bool) {
        self.show_multi_node_bounds = show;
    }

    /// Geometric bounding box over all 2+ sub-selected nodes in document
    /// space. Returns `None` if fewer than 2 distinct nodes are selected.
    #[must_use]
    pub fn multi_node_bounds(&self) -> Option<petunia_core::Rect> {
        let nodes = self.selection.sub.nodes();
        if nodes.len() < 2 {
            return None;
        }
        let mut bounds: Option<petunia_core::Rect> = None;
        for node in nodes {
            let scene_node = self.document.scene.get_node(node.object)?;
            let path = scene_node.item_path()?;
            let contour = path.contours.get(node.contour as usize)?;
            let path_node = contour.nodes.get(node.node as usize)?;
            let doc_point = scene_node.transform.transform_point(path_node.point);
            let slot = petunia_core::Rect::new(doc_point.x, doc_point.y, 0.0, 0.0);
            bounds = Some(match bounds {
                Some(existing) => existing.union(slot),
                None => slot,
            });
        }
        bounds
    }

    /// Discrete multi-node bounding box overlay primitive (Decision C).
    #[must_use]
    pub fn multi_node_bounds_overlay(&self) -> Option<crate::tools::OverlayPrimitive> {
        if !self.show_multi_node_bounds {
            return None;
        }
        let bounds = self.multi_node_bounds()?;
        let view = self.view_transform();
        let min = view.doc_to_view(bounds.min());
        let max = view.doc_to_view(bounds.max());
        Some(crate::tools::OverlayPrimitive::Rect {
            x: min.x,
            y: min.y,
            width: (max.x - min.x).max(1.0),
            height: (max.y - min.y).max(1.0),
        })
    }

    /// Whether an object may be edited: exists, visible and unlocked.
    #[must_use]
    pub fn is_editable(&self, id: ObjectId) -> bool {
        std::iter::once(id)
            .chain(self.document.scene.ancestors(id))
            .all(|id| {
                self.document
                    .scene
                    .get_node(id)
                    .is_some_and(|node| node.visible && !node.locked)
            })
    }

    /// Select a validated path node from a semantic keyboard control.
    /// Selection and context are transient; no document revision is created.
    pub fn select_node(&mut self, node: crate::NodeId, additive: bool) -> Result<()> {
        if self.interaction_active() || !self.is_editable(node.object) {
            return Err(crate::UiError::State(
                "Nó indisponível durante esta operação".into(),
            ));
        }
        let valid = self
            .document
            .scene
            .get_node(node.object)
            .and_then(|scene| scene.item_path())
            .and_then(|path| path.contours.get(node.contour as usize))
            .and_then(|contour| contour.nodes.get(node.node as usize))
            .is_some();
        if !valid {
            return Err(crate::UiError::State("Nó inexistente".into()));
        }
        if !additive {
            self.selection.set_objects(vec![node.object]);
            self.selection.sub.clear();
        } else {
            self.selection.add_objects([node.object]);
        }
        self.selection.sub.select_node(node);
        self.on_enter();
        self.sync_layers();
        Ok(())
    }

    /// Double-click: pick an object and open its semantic context.
    pub fn on_double_click(&mut self, view_point: Point) {
        let Some(hit) = self.hit_test(view_point).into_iter().next() else {
            return;
        };
        let object = match hit {
            HitTarget::Fill { object } => object,
            _ => return,
        };
        let context = match self.item_kind(object) {
            ItemKind::Group => EditContext::Group { group: object },
            ItemKind::Shape => EditContext::Shape { target: object },
            ItemKind::Text => EditContext::Text { target: object },
            ItemKind::Symbol => EditContext::Symbol { target: object },
            _ => EditContext::Vector {
                targets: vec![object],
                operation: VectorOperation::Node,
            },
        };
        self.contexts.push(context);
    }

    /// Dispatch a user action through the session.
    pub fn dispatch_action(&mut self, action: UserAction) -> Result<ToolResponse> {
        match action {
            UserAction::SelectTool(tool) => {
                self.select_tool(tool);
                Ok(ToolResponse::Idle)
            }
            UserAction::Undo => {
                self.history.undo(&mut self.document)?;
                self.sync_layers();
                Ok(ToolResponse::Idle)
            }
            UserAction::Redo => {
                self.history.redo(&mut self.document)?;
                self.sync_layers();
                Ok(ToolResponse::Idle)
            }
            UserAction::SmartDelete(mode) => self.smart_delete(mode),
            UserAction::Nudge { dx, dy } => self.nudge(dx, dy),
            // Hover never mutates: a tooltip is Session State at most.
            UserAction::HoverTooltip(_) => Ok(ToolResponse::Idle),
            UserAction::Pointer(pointer) => {
                let response = self.handle_pointer(pointer)?;
                self.sync_layers();
                Ok(response)
            }
        }
    }

    /// Smart Delete (ADR-0012 D7) on the selected nodes.
    ///
    /// Each selected node is deleted in one transaction per gesture,
    /// and a fit above the tolerance reports `NeedsConfirmation`
    /// instead of committing a worse curve.
    pub fn smart_delete(&mut self, mode: SmartDeleteMode) -> Result<ToolResponse> {
        let nodes = self.selection.sub.nodes().to_vec();
        if nodes.is_empty() {
            return Ok(ToolResponse::Failed(
                "Smart Delete precisa de nodes selecionados".to_string(),
            ));
        }
        let mut operations = Vec::new();
        let mut deviations = Vec::new();
        // Later deletions must not shift the indexes of earlier ones,
        // so the highest index goes first.
        let mut ordered = nodes;
        ordered.sort_by(|left, right| {
            left.object
                .cmp(&right.object)
                .then(left.contour.cmp(&right.contour))
                .then(left.node.cmp(&right.node).reverse())
        });
        for node in &ordered {
            let Some(scene_node) = self.document.scene.get_node(node.object) else {
                continue;
            };
            let Some(path) = scene_node.item_path() else {
                continue;
            };
            let Some(contour) = path.contours.get(node.contour as usize) else {
                continue;
            };
            let outcome = delete_node(contour, node.node as usize, mode, 1.0);
            match outcome {
                SmartDeleteOutcome::Applied { nodes, max_error } => {
                    deviations.push(max_error);
                    let mut path = path.clone();
                    path.contours[node.contour as usize].nodes = nodes;
                    operations.push(DocumentOp::ReplacePath {
                        object: node.object,
                        path,
                    });
                }
                SmartDeleteOutcome::NeedsConfirmation {
                    max_error,
                    tolerance,
                } => {
                    return Ok(ToolResponse::Failed(format!(
                        "erro máximo {max_error:.3} excede a tolerância {tolerance:.3}: escolha Hard Delete ou cancele"
                    )));
                }
                SmartDeleteOutcome::Refused(reason) => {
                    return Ok(ToolResponse::Failed(refusal_message(reason)));
                }
            }
        }
        if operations.is_empty() {
            return Ok(ToolResponse::Failed(
                "nenhum node elegível para deleção".to_string(),
            ));
        }
        self.commit_request(
            TransactionRequest {
                command_id: CommandId::new_v4(),
                operations,
                merge_key: None,
            },
            HistoryDescription::DeleteObjects,
        )?;
        let worst = deviations
            .iter()
            .fold(0.0_f64, |worst, error| worst.max(*error));
        Ok(ToolResponse::Status(format!(
            "Smart Delete aplicado, erro máximo {worst:.3}"
        )))
    }

    /// Preview a Smart Delete without committing: ghost points of the
    /// candidate curve plus its maximum deviation and whether it fits
    /// the tolerance. What the preview shows is exactly what the
    /// commit would produce.
    pub fn smart_delete_preview(
        &self,
        node: NodeId,
    ) -> std::result::Result<SmartDeleteGhost, EngineError> {
        let scene_node = self
            .document
            .scene
            .get_node(node.object)
            .ok_or_else(|| EngineError::Execution(format!("object {} is gone", node.object)))?;
        let path = scene_node.item_path().ok_or_else(|| {
            EngineError::Execution(format!("object {} is not a path", node.object))
        })?;
        let contour = path
            .contours
            .get(node.contour as usize)
            .ok_or_else(|| EngineError::Execution(format!("contour {} is gone", node.contour)))?;
        let preview = preview_delete(contour, node.node as usize, 1.0).map_err(|refusal| {
            EngineError::Execution(format!("smart delete refused: {refusal:?}"))
        })?;
        let band = petunia_core::Tolerance::new(0.25).unwrap_or(petunia_core::Tolerance(0.25));
        let candidate = petunia_core::Contour {
            id: petunia_core::ContourId::new_v4(),
            nodes: preview.nodes.clone(),
            closed: contour.closed,
        };
        let ghost = flatten_contour(&candidate, band);
        Ok(SmartDeleteGhost {
            points: ghost,
            max_error: preview.max_error,
            within_tolerance: preview.within_tolerance,
        })
    }

    /// Move the selection by document units, independent of zoom
    /// (decision F of A-F, ADR-0012 D3).
    pub fn nudge(&mut self, dx: f64, dy: f64) -> Result<ToolResponse> {
        if self.selection.is_empty() {
            return Ok(ToolResponse::Failed(
                "nada selecionado para mover".to_string(),
            ));
        }
        self.move_selection(dx, dy)?;
        Ok(ToolResponse::Status(format!(
            "seleção movida para ({dx:.0}, {dy:.0}) em unidades do documento"
        )))
    }

    /// Run one pointer event through the active tool.
    pub fn handle_pointer(&mut self, pointer: PointerEvent) -> Result<ToolResponse> {
        let (position, pressure) = match pointer {
            PointerEvent::Down { position, pressure }
            | PointerEvent::Move { position, pressure } => (position, pressure),
            PointerEvent::Up { position } => (position, 1.0),
        };
        let mut sample = PointerSample::new(position);
        sample.pressure = pressure;
        self.dispatch_pointer_sample(pointer, sample)
    }

    /// Preserve modifier and stylus information received at the Qt boundary.
    pub fn dispatch_pointer_sample(
        &mut self,
        pointer: PointerEvent,
        sample: PointerSample,
    ) -> Result<ToolResponse> {
        if !sample.position.x.is_finite()
            || !sample.position.y.is_finite()
            || !sample.pressure.is_finite()
            || !(0.0..=1.0).contains(&sample.pressure)
        {
            return Err(crate::UiError::State(
                "coordenadas ou pressão inválidas".into(),
            ));
        }
        let is_end = matches!(pointer, PointerEvent::Up { .. });
        let response = match self.active_tool {
            ToolKind::NodeEdit => {
                let mut tool = self.tools.node.clone();
                let response = match pointer {
                    PointerEvent::Down { .. } => tool.begin(self, &sample),
                    PointerEvent::Move { .. } => tool.update(self, &sample),
                    PointerEvent::Up { .. } => tool.end(self, &sample),
                };
                self.tools.node = tool;
                response
            }
            ToolKind::Pen => {
                let mut tool = self.tools.pen.clone();
                let response = match pointer {
                    PointerEvent::Down { .. } => tool.begin(self, &sample),
                    PointerEvent::Move { .. } => tool.update(self, &sample),
                    PointerEvent::Up { .. } => tool.end(self, &sample),
                };
                self.tools.pen = tool;
                response
            }
            _ => {
                let mut tool = self.tools.select.clone();
                let response = match pointer {
                    PointerEvent::Down { .. } => tool.begin(self, &sample),
                    PointerEvent::Move { .. } => tool.update(self, &sample),
                    PointerEvent::Up { .. } => tool.end(self, &sample),
                };
                self.tools.select = tool;
                response
            }
        };
        if is_end {
            self.captured = false;
        }
        // The session is the only writer: the tool asks declaratively,
        // and the session applies the change to Session State or commits.
        match &response {
            ToolResponse::Selection(delta) => self.apply_selection(delta.clone()),
            ToolResponse::Commit(operations) => {
                self.commit_request(
                    TransactionRequest {
                        command_id: CommandId::new_v4(),
                        operations: operations.clone(),
                        merge_key: None,
                    },
                    HistoryDescription::EditObjects,
                )?;
            }
            _ => {}
        }
        Ok(response)
    }

    /// Explicitly finish a staged open pen path as one transaction.
    pub fn finish_pen(&mut self) -> Result<ToolResponse> {
        let mut tool = self.tools.pen.clone();
        let response = tool.finish(self);
        if let ToolResponse::Commit(operations) = &response {
            self.commit_request(
                TransactionRequest {
                    command_id: CommandId::new_v4(),
                    operations: operations.clone(),
                    merge_key: None,
                },
                HistoryDescription::InsertObjects,
            )?;
        }
        self.tools.pen = tool;
        self.sync_layers();
        Ok(response)
    }

    /// Apply one selection delta. Selecting a sub-item keeps its object
    /// in the object selection, per D2.
    fn apply_selection(&mut self, delta: SelectionDelta) {
        match delta {
            SelectionDelta::ReplaceObjects(objects) => self.selection.set_objects(objects),
            SelectionDelta::ToggleObject(object) => self.selection.toggle_object(object),
            SelectionDelta::ClearObjects => self.selection.clear(),
            SelectionDelta::SelectNode(node) => {
                self.selection.add_objects([node.object]);
                self.selection.sub.select_node(node);
            }
            SelectionDelta::ToggleNode(node) => {
                self.selection.add_objects([node.object]);
                self.selection.sub.toggle_node(node);
            }
            SelectionDelta::SelectSegment(segment) => {
                self.selection.add_objects([segment.object]);
                self.selection.sub.select_segment(segment);
            }
            SelectionDelta::ToggleHandle(handle) => {
                self.selection.add_objects([handle.object]);
                self.selection.sub.toggle_handle(handle);
            }
            SelectionDelta::ClearSub => self.selection.sub.clear(),
        }
    }

    /// Headless render: compiles, reports degraded primitives, draws.
    pub fn render_headless(
        &mut self,
        options: &RenderOptions,
    ) -> std::result::Result<(Vec<u8>, RenderStats), EngineError> {
        if !self.view.scale.is_finite()
            || self.view.scale <= 0.0
            || !self.view.dpr.is_finite()
            || self.view.dpr <= 0.0
            || !self.view.rotation.is_finite()
            || !self.view.pan_x.is_finite()
            || !self.view.pan_y.is_finite()
            || !self.view.viewport.width.is_finite()
            || !self.view.viewport.height.is_finite()
        {
            return Err(EngineError::Execution("invalid viewport transform".into()));
        }
        let (snapshot, warnings) = self.compile_snapshot(options.quality);
        for warning in &warnings {
            eprintln!("[render] {}: {}", warning.source, warning.message);
        }
        let width = (self.view.viewport.width * self.view.dpr).ceil().max(1.0);
        let height = (self.view.viewport.height * self.view.dpr).ceil().max(1.0);
        if width > u32::MAX as f64 || height > u32::MAX as f64 || width * height > (64 << 20) as f64
        {
            return Err(EngineError::Limit(
                "viewport exceeds render target budget".into(),
            ));
        }
        let mut frame = compile::headless_page_frame(
            snapshot,
            self.active_page,
            width as u32,
            height as u32,
            self.view.effective_scale(),
        )
        .ok_or_else(|| EngineError::Execution("active page missing from snapshot".into()))?;
        frame.view.rotation = self.view.rotation;
        frame.view.offset_x = self.view.pan_x * self.view.dpr;
        frame.view.offset_y = self.view.pan_y * self.view.dpr;
        let mut renderer = SoftwareRenderer::new(64 << 20, 64 << 20);
        renderer
            .render(&frame, options)
            .map_err(|error| EngineError::Execution(error.to_string()))
    }

    /// Hit-test a view point, ordered by D1 precedence.
    ///
    /// `Handle > Node > Segment > Fill`, ties by z-order (topmost
    /// first), which is the same order the semantic tree exposes.
    #[must_use]
    pub fn hit_test(&self, view_point: Point) -> Vec<HitTarget> {
        use petunia_engine::spatial::{self, HitTestMode, HitTestRequest, RStarIndex};
        let view = self.view_transform();
        if !view.scale.is_finite() || view.scale <= 0.0 || !view.rotation.is_finite() {
            return Vec::new();
        }
        let document_point = view.view_to_doc(view_point);
        let mut handles = Vec::new();
        let mut nodes = Vec::new();
        let mut segments = Vec::new();
        for id in spatial::paint_order_on_page(&self.document, self.active_page)
            .into_iter()
            .rev()
        {
            if !self.is_editable(id) || !self.selection.objects().contains(&id) {
                continue;
            }
            let Some(node) = self.document.scene.get_node(id) else {
                continue;
            };
            let Some(path) = node.item_path() else {
                continue;
            };
            let Some(world) = self.document.scene.world_transform(id) else {
                continue;
            };
            let mut screen_path = path.clone();
            for contour in &mut screen_path.contours {
                for anchor in &mut contour.nodes {
                    anchor.point = view.doc_to_view(world.transform_point(anchor.point));
                    anchor.handle_in = anchor
                        .handle_in
                        .map(|point| view.doc_to_view(world.transform_point(point)));
                    anchor.handle_out = anchor
                        .handle_out
                        .map(|point| view.doc_to_view(world.transform_point(point)));
                }
            }
            let (anchor, handle) = node_hit(&screen_path, view_point, 8.0);
            if let Some(((contour, node), handle)) = handle {
                handles.push(HitTarget::Handle {
                    object: id,
                    contour,
                    node,
                    handle,
                });
            } else if let Some((contour, node)) = anchor {
                nodes.push(HitTarget::Node {
                    object: id,
                    contour,
                    node,
                });
            } else {
                for (ci, contour) in screen_path.contours.iter().enumerate() {
                    let count = contour.nodes.len();
                    let segment_count = if contour.closed {
                        count
                    } else {
                        count.saturating_sub(1)
                    };
                    for from in 0..segment_count {
                        let to = (from + 1) % count;
                        let mut piece = petunia_core::Contour::new(false);
                        piece.nodes = vec![contour.nodes[from].clone(), contour.nodes[to].clone()];
                        let points = petunia_engine::geometry::bezier::try_flatten_contour(
                            &piece,
                            petunia_core::Tolerance(0.15),
                            65536,
                        )
                        .unwrap_or_default();
                        if points
                            .windows(2)
                            .any(|pair| segment_distance(view_point, pair[0], pair[1]) <= 8.0)
                        {
                            segments.push(HitTarget::Segment {
                                object: id,
                                contour: ci as u32,
                                from: from as u32,
                                to: to as u32,
                            });
                            break;
                        }
                    }
                }
            }
        }
        let (snapshot, _) = self.compile_snapshot(RenderQuality::Authoring);
        let index = RStarIndex::bulk_load(spatial::entries_for_snapshot(&snapshot));
        let (sin, cos) = view.rotation.sin_cos();
        let matrix = petunia_core::Transform2D {
            a: cos * view.scale,
            b: sin * view.scale,
            c: -sin * view.scale,
            d: cos * view.scale,
            tx: view.offset_x,
            ty: view.offset_y,
        };
        let fills = spatial::hit_test_with_snapshot(
            &self.document,
            &snapshot,
            &index,
            HitTestRequest {
                page: self.active_page,
                point_document: document_point,
                tolerance_px: 8.0,
                mode: HitTestMode::Any,
            },
            matrix,
        );
        handles.extend(nodes);
        handles.extend(segments);
        handles.extend(
            fills
                .into_iter()
                .filter_map(|hit| self.scoped_object(hit.object))
                .fold(Vec::new(), |mut objects, id| {
                    if !objects.contains(&id) {
                        objects.push(id);
                    }
                    objects
                })
                .into_iter()
                .map(|object| HitTarget::Fill { object }),
        );
        handles
    }

    fn scoped_object(&self, id: ObjectId) -> Option<ObjectId> {
        let parent = match self.contexts.current() {
            EditContext::Scene => None,
            EditContext::Group { group } => Some(*group),
            EditContext::Vector { targets, .. } => return targets.contains(&id).then_some(id),
            EditContext::Shape { target }
            | EditContext::Text { target }
            | EditContext::Symbol { target } => return (*target == id).then_some(id),
        };
        let mut cursor = id;
        loop {
            match self.document.scene.parent_of(cursor)? {
                ParentRef::Page(page) => {
                    return (parent.is_none() && page == self.active_page).then_some(cursor)
                }
                ParentRef::Object(object) if Some(object) == parent => return Some(cursor),
                ParentRef::Object(object) => cursor = object,
            }
        }
    }

    /// Evaluated world bounds shared by rendering and spatial selection.
    pub fn object_bounds(&self, id: ObjectId) -> Option<petunia_core::Rect> {
        let (snapshot, _) = self.compile_snapshot(RenderQuality::Authoring);
        let descendants: std::collections::HashSet<_> = std::iter::once(id)
            .chain(self.document.scene.descendants(id))
            .collect();
        petunia_engine::spatial::entries_for_snapshot(&snapshot)
            .into_iter()
            .filter(|entry| descendants.contains(&entry.object))
            .map(|entry| entry.bounds)
            .reduce(|left, right| left.union(right))
    }

    fn item_kind(&self, id: ObjectId) -> ItemKind {
        match self.document.scene.get_node(id).map(|node| &node.item) {
            Some(SceneItem::Path(_)) => ItemKind::Path,
            Some(SceneItem::Group(_)) => ItemKind::Group,
            Some(SceneItem::Text(_)) => ItemKind::Text,
            Some(SceneItem::Shape(_)) => ItemKind::Shape,
            Some(SceneItem::SymbolInstance(_)) => ItemKind::Symbol,
            _ => ItemKind::Other,
        }
    }

    fn view_transform(&self) -> ViewTransform {
        ViewTransform {
            scale: self.view.scale,
            rotation: self.view.rotation,
            offset_x: self.view.pan_x,
            offset_y: self.view.pan_y,
        }
    }

    /// Layers panel, mirroring the scene hierarchy.
    #[must_use]
    pub fn layers(&self) -> &LayersPanel {
        &self.layers
    }

    /// Rebuild the panel when the document moved on. Focus, collapse
    /// and selection survive by identity.
    pub fn sync_layers(&mut self) {
        let revision = self.revision();
        let selected = self.selection.objects().to_vec();
        if revision == self.layers_revision && selected == self.layers_selection {
            return;
        }
        let rows = self.build_layer_rows();
        self.layers.rebuild(rows, &selected, revision.0);
        self.layers_revision = revision;
        self.layers_selection = selected;
    }

    /// Keyboard intent on the layers panel. Arrows move focus, Enter
    /// selects the focused row, Space toggles its visibility through
    /// one transaction.
    pub fn layer_key(&mut self, key: LayerKey) -> Result<ToolResponse> {
        self.sync_layers();
        match key {
            LayerKey::Enter => {
                let Some(id) = self.layers.focused_id() else {
                    return Ok(ToolResponse::Idle);
                };
                self.selection.set_objects(vec![id]);
                self.sync_layers();
                Ok(ToolResponse::Selection(SelectionDelta::ReplaceObjects(
                    vec![id],
                )))
            }
            LayerKey::Space => self.toggle_focused_visibility(),
            _ => {
                self.layers.key(key);
                Ok(ToolResponse::Idle)
            }
        }
    }

    /// Toggle visibility of the focused object row in one transaction.
    fn toggle_focused_visibility(&mut self) -> Result<ToolResponse> {
        let Some(id) = self.layers.focused_id() else {
            return Ok(ToolResponse::Idle);
        };
        let Some(node) = self.document.scene.get_node(id) else {
            return Ok(ToolResponse::Failed("row points nowhere".to_string()));
        };
        let visible = !node.visible;
        self.commit_request(
            TransactionRequest {
                command_id: CommandId::new_v4(),
                operations: vec![DocumentOp::SetVisibility {
                    object: id,
                    visible,
                }],
                merge_key: None,
            },
            HistoryDescription::SetVisibility,
        )?;
        self.sync_layers();
        Ok(ToolResponse::Status(if visible {
            "visível".to_string()
        } else {
            "oculto".to_string()
        }))
    }

    /// Rows in panel order: pages, then depth-first roots.
    fn build_layer_rows(&self) -> Vec<LayerRow> {
        let mut rows = Vec::new();
        for page_id in self.document.scene.page_ids() {
            let Some(page) = self.document.pages.get(page_id) else {
                continue;
            };
            let roots = self.document.scene.page_roots(page_id).unwrap_or(&[]);
            rows.push(LayerRow {
                id: None,
                page: page_id,
                name: page.name.clone(),
                kind: LayerKind::Page,
                depth: 0,
                visible: true,
                locked: false,
                child_count: roots.len(),
                position: 1,
                siblings: 1,
                selected: false,
            });
            let count = roots.len();
            for (index, id) in roots.iter().copied().enumerate() {
                self.push_layer_rows(id, page_id, 1, index + 1, count, &mut rows);
            }
        }
        rows
    }

    fn push_layer_rows(
        &self,
        id: ObjectId,
        page: petunia_core::PageId,
        depth: usize,
        position: usize,
        siblings: usize,
        rows: &mut Vec<LayerRow>,
    ) {
        let Some(node) = self.document.scene.get_node(id) else {
            return;
        };
        let (kind, child_count) = match &node.item {
            SceneItem::Group(children) => (LayerKind::Group, children.len()),
            SceneItem::Path(_) => (LayerKind::Path, 0),
            SceneItem::Shape(_) => (LayerKind::Shape, 0),
            SceneItem::Text(_) => (LayerKind::Text, 0),
            SceneItem::Image(_) => (LayerKind::Image, 0),
            SceneItem::PixelLayer(_) => (LayerKind::Pixel, 0),
            SceneItem::Trace(_) => (LayerKind::Trace, 0),
            SceneItem::GeneratedVector(_) => (LayerKind::Generated, 0),
            SceneItem::SymbolInstance(_) => (LayerKind::Symbol, 0),
        };
        rows.push(LayerRow {
            id: Some(id),
            page,
            name: node.name.clone(),
            kind,
            depth,
            visible: node.visible,
            locked: node.locked,
            child_count,
            position,
            siblings,
            selected: false,
        });
        if let SceneItem::Group(children) = &node.item {
            let count = children.len();
            for (index, child) in children.iter().copied().enumerate() {
                self.push_layer_rows(child, page, depth + 1, index + 1, count, rows);
            }
        }
    }

    /// Label for the context bar, mirroring D2.
    #[must_use]
    pub fn context_label(&self) -> String {
        match self.contexts.current() {
            EditContext::Scene => "Scene".to_string(),
            EditContext::Group { group } => format!("Grupo {group}"),
            EditContext::Vector { operation, .. } => {
                format!("Vector Edit · {}", operation.as_str())
            }
            EditContext::Shape { target } => format!("Shape {target}"),
            EditContext::Text { target } => format!("Texto {target}"),
            EditContext::Symbol { target } => format!("Símbolo {target}"),
        }
    }

    /// Insert a path at the document root as one atomic operation.
    pub fn insert_path(
        &mut self,
        name: impl Into<String>,
        path: VectorPath,
    ) -> std::result::Result<(), EngineError> {
        let page = self.active_page;
        let node = SceneNode::new_path(name, path, ParentRef::Page(page));
        let request = TransactionRequest {
            command_id: CommandId::new_v4(),
            operations: vec![DocumentOp::InsertRoot {
                index: self
                    .document
                    .scene
                    .page_roots(self.active_page)
                    .unwrap_or_default()
                    .len(),
                node: Box::new(node),
            }],
            merge_key: None,
        };
        self.commit_request(request, HistoryDescription::InsertObjects)
    }

    /// Adjust the selection through one engine transaction.
    pub fn move_selection(&mut self, dx: f64, dy: f64) -> std::result::Result<bool, EngineError> {
        let selection: Vec<ObjectId> = self.selection.objects().to_vec();
        if selection.is_empty() || (dx == 0.0 && dy == 0.0) {
            return Ok(false);
        }
        let mut operations = Vec::with_capacity(selection.len());
        for id in &selection {
            if !self.is_editable(*id) {
                continue;
            }
            operations.push(DocumentOp::MoveObjects {
                object: *id,
                dx,
                dy,
            });
        }
        if operations.is_empty() {
            return Ok(false);
        }
        self.commit_request(
            TransactionRequest {
                command_id: CommandId::new_v4(),
                operations,
                merge_key: None,
            },
            HistoryDescription::MoveObjects,
        )?;
        Ok(true)
    }

    /// Adapter entry point for confirmed edits; tools and the GUI share the writer.
    pub fn apply_edit(
        &mut self,
        request: TransactionRequest,
        description: HistoryDescription,
    ) -> Result<()> {
        self.commit_request(request, description)?;
        Ok(())
    }

    /// Commit one request atomically through the session.
    pub fn commit_request(
        &mut self,
        request: TransactionRequest,
        description: HistoryDescription,
    ) -> std::result::Result<(), EngineError> {
        if request.operations.is_empty() {
            return Ok(());
        }
        let prepared = prepare_transaction(&self.document, request, self.revision())
            .map_err(|error| EngineError::Execution(error.to_string()))?;
        // The session returns `EngineError`, so the new revision is
        // redundant to the caller; the caller can read `revision()`.
        self.history
            .commit(&mut self.document, prepared, description)
            .map(|_| ())?;
        self.sync_layers();
        Ok(())
    }
}

impl ToolServices for StudioSession {
    fn hit_test(&self, view_point: Point) -> Vec<HitTarget> {
        StudioSession::hit_test(self, view_point)
    }

    fn snap(&self, document_point: Point) -> (Point, Option<String>) {
        use petunia_engine::spatial::*;
        if !self.snap_settings.enabled || !self.view.scale.is_finite() || self.view.scale <= 0.0 {
            self.snap_latch.set(None);
            return (document_point, None);
        }
        let anchor = document_point;
        let scale = self.view.scale;
        let tolerance = petunia_core::Tolerance(self.snap_settings.release_px / scale);
        let guides: Vec<_> = self
            .document
            .guides
            .iter()
            .filter_map(|(_, guide)| match guide.scope {
                petunia_core::GuideScope::Document => Some(*guide),
                petunia_core::GuideScope::Page(page) if page == self.active_page => Some(*guide),
                _ => None,
            })
            .collect();
        let mut candidates = guide_candidates(&guides, &[anchor], tolerance, scale);
        for (_, grid) in self.document.grids.iter() {
            if !matches!(grid.scope, petunia_core::GridScope::Document)
                && grid.scope != petunia_core::GridScope::Page(self.active_page)
            {
                continue;
            }
            if let petunia_core::GridSpec::Affine(spec) = grid.spec {
                candidates.extend(affine_grid_candidates(
                    &spec,
                    grid.origin,
                    &[anchor],
                    tolerance,
                    scale,
                ));
            }
        }
        let (snapshot, _) = self.compile_snapshot(RenderQuality::Authoring);
        let bounds: Vec<_> = entries_for_snapshot(&snapshot)
            .into_iter()
            .filter(|entry| self.is_editable(entry.object))
            .map(|entry| (entry.object, entry.bounds))
            .collect();
        let excluded: Vec<_> = self
            .selection
            .objects()
            .iter()
            .flat_map(|id| std::iter::once(*id).chain(self.document.scene.descendants(*id)))
            .collect();
        candidates.extend(bounds_candidates(
            &bounds,
            &excluded,
            &[anchor],
            tolerance,
            scale,
        ));
        let request = SnapRequest {
            page: self.active_page,
            moving: MovingGeometry {
                excluded,
                anchors: vec![anchor],
            },
            proposed: TransformDelta::default(),
            constraint: SnapConstraint::Free,
            settings: self.snap_settings.clone(),
            view_scale: scale,
            previous: self.snap_latch.get(),
        };
        let result = solve(&request, &candidates);
        self.snap_latch.set(result.latch);
        let corrected = Point::new(
            anchor.x + result.corrected.dx,
            anchor.y + result.corrected.dy,
        );
        (
            corrected,
            result
                .matches
                .first()
                .map(|hit| format!("{:?}", hit.target)),
        )
    }

    fn marquee_select(&self, min: Point, max: Point) -> Vec<ObjectId> {
        let (snapshot, _) = self.compile_snapshot(RenderQuality::Authoring);
        let mut bounds = BTreeMap::<ObjectId, petunia_core::Rect>::new();
        for entry in petunia_engine::spatial::entries_for_snapshot(&snapshot) {
            if !self.is_editable(entry.object) {
                continue;
            }
            if let Some(id) = self.scoped_object(entry.object) {
                bounds
                    .entry(id)
                    .and_modify(|old| *old = old.union(entry.bounds))
                    .or_insert(entry.bounds);
            }
        }
        petunia_engine::spatial::paint_order_on_page(&self.document, self.active_page)
            .into_iter()
            .filter(|id| {
                bounds.get(id).is_some_and(|bounds| {
                    [
                        bounds.min(),
                        Point::new(bounds.x + bounds.width, bounds.y),
                        bounds.max(),
                        Point::new(bounds.x, bounds.y + bounds.height),
                    ]
                    .into_iter()
                    .map(|point| self.view_transform().doc_to_view(point))
                    .all(|point| {
                        point.x >= min.x && point.y >= min.y && point.x <= max.x && point.y <= max.y
                    })
                })
            })
            .collect()
    }
}

impl ToolSession for StudioSession {
    fn services(&self) -> &dyn ToolServices {
        self
    }

    fn view(&self) -> ViewTransform {
        self.view_transform()
    }

    fn selected_objects(&self) -> Vec<ObjectId> {
        self.selection.objects().to_vec()
    }

    fn select_objects(&mut self, objects: Vec<ObjectId>) {
        self.selection.set_objects(objects);
    }

    fn toggle_object(&mut self, id: ObjectId) {
        self.selection.toggle_object(id);
    }

    fn select_node(&mut self, id: NodeId) {
        self.selection.sub.select_node(id);
    }

    fn toggle_node(&mut self, id: NodeId) {
        self.selection.sub.toggle_node(id);
    }

    fn toggle_handle(&mut self, id: HandleId) {
        self.selection.sub.toggle_handle(id);
    }

    fn clear_sub_selection(&mut self) {
        self.selection.sub.clear();
    }

    fn selected_nodes(&self) -> Vec<NodeId> {
        self.selection.sub.nodes().to_vec()
    }

    fn path_snapshot(&self, object: ObjectId) -> Option<VectorPath> {
        self.document
            .scene
            .get_node(object)
            .and_then(|node| node.item_path().cloned())
    }

    fn default_page(&self) -> petunia_core::PageId {
        self.active_page
    }

    fn root_count(&self) -> usize {
        self.document
            .scene
            .page_roots(self.active_page)
            .unwrap_or_default()
            .len()
    }

    fn is_editable(&self, id: ObjectId) -> bool {
        StudioSession::is_editable(self, id)
    }

    fn capture(&mut self) {
        self.captured = true;
    }

    fn release(&mut self) {
        self.captured = false;
    }

    fn is_captured(&self) -> bool {
        self.captured
    }
}

/// A contour/node coordinate inside a path.
type ContourNode = (u32, u32);

/// Human message for a Smart Delete refusal.
fn refusal_message(reason: petunia_engine::geometry::SmartDeleteRefusal) -> String {
    use petunia_engine::geometry::SmartDeleteRefusal;
    match reason {
        SmartDeleteRefusal::IndexOutOfRange => "node fora do contorno".to_string(),
        SmartDeleteRefusal::TooFewNodes => {
            "o contorno ficaria com poucos nodes para manter a forma".to_string()
        }
        SmartDeleteRefusal::NonFiniteGeometry => "geometria inválida no contorno".to_string(),
    }
}

/// Node and handle candidates for one path at a local point.
fn node_hit(
    path: &VectorPath,
    local: Point,
    tolerance: f64,
) -> (Option<ContourNode>, Option<(ContourNode, HandleRef)>) {
    let mut node_candidate = None;
    let mut handle_candidate = None;

    for (contour_index, contour) in path.contours.iter().enumerate() {
        for (node_index, node) in contour.nodes.iter().enumerate() {
            let at_in = node
                .handle_in
                .is_some_and(|handle| distance(handle, local) <= tolerance);
            let at_out = node
                .handle_out
                .is_some_and(|handle| distance(handle, local) <= tolerance);

            if at_in || at_out {
                let handle = if at_in {
                    crate::context::HandleRef::In
                } else {
                    crate::context::HandleRef::Out
                };
                if handle_candidate.is_none() {
                    handle_candidate = Some(((contour_index as u32, node_index as u32), handle));
                }
                continue;
            }

            let at_node = distance(node.point, local) <= tolerance;
            if at_node && node_candidate.is_none() {
                node_candidate = Some((contour_index as u32, node_index as u32));
            }
        }
    }

    (node_candidate, handle_candidate)
}

fn segment_distance(point: Point, a: Point, b: Point) -> f64 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let length = dx * dx + dy * dy;
    let t = if length > 0.0 {
        (((point.x - a.x) * dx + (point.y - a.y) * dy) / length).clamp(0.0, 1.0)
    } else {
        0.0
    };
    distance(point, Point::new(a.x + t * dx, a.y + t * dy))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn selection_changed(response: &ToolResponse) -> bool {
        matches!(response, ToolResponse::Selection(_))
    }

    fn session_with_rect() -> StudioSession {
        let mut session = StudioSession::new("headless");
        session
            .insert_path("box", VectorPath::rect(0.0, 0.0, 100.0, 100.0))
            .expect("insert");
        let object = session
            .document()
            .scene
            .root_order()
            .first()
            .copied()
            .expect("object");
        session.selection.set_objects(vec![object]);
        session
    }

    fn center_of_object(session: &StudioSession, object: ObjectId) -> Point {
        let node = session.document().scene.get_node(object).expect("node");
        node.transform.transform_point(Point::new(50.0, 50.0))
    }

    fn center_of(session: &StudioSession) -> Point {
        center_of_object(session, session.selection.single().expect("selected"))
    }

    #[test]
    fn select_picks_object_and_moves_it_in_one_transaction() {
        let mut session = session_with_rect();
        let object = session.selection.single().expect("selected");

        // Center of the object, computed before selection changes.
        let center = center_of_object(&session, object);

        // A click on empty background clears the selection.
        let response = session
            .dispatch_action(UserAction::Pointer(PointerEvent::Down {
                position: Point::new(150.0, 150.0),
                pressure: 1.0,
            }))
            .expect("dispatch");
        assert!(selection_changed(&response), "{response:?}");
        assert!(session.selection.is_empty());

        // A click on the object selects it again.
        let response = session
            .dispatch_action(UserAction::Pointer(PointerEvent::Down {
                position: center,
                pressure: 1.0,
            }))
            .expect("dispatch");
        assert!(selection_changed(&response), "{response:?}");
        assert_eq!(session.selection.single(), Some(object));

        // Moving the selection is one transaction, with revision +1.
        let revision = session.revision();
        assert!(session.move_selection(10.0, 20.0).expect("moves"));
        assert_eq!(session.revision().0, revision.0 + 1);

        // Undo restores exactly one step.
        session.dispatch_action(UserAction::Undo).expect("undo");
        let restored = session.document().scene.get_node(object).expect("node");
        assert_eq!(restored.transform.tx, 0.0);
        assert_eq!(restored.transform.ty, 0.0);
    }

    #[test]
    fn escape_never_commits_and_context_unwinds() {
        let mut session = session_with_rect();
        let center = center_of(&session);
        session
            .dispatch_action(UserAction::Pointer(PointerEvent::Down {
                position: center,
                pressure: 1.0,
            }))
            .expect("select");
        let revision = session.revision();
        session.on_enter();
        assert!(session.contexts().in_vector());
        // Escape while in Vector Edit unwinds one level, no commit.
        let _ = session.on_escape();
        assert!(!session.contexts().in_vector());
        assert_eq!(session.revision(), revision);
    }

    #[test]
    fn locked_path_is_not_editable() {
        let mut session = session_with_rect();
        let object = session.selection.single().expect("selected");
        if let Some(node) = session.document.scene.get_node_mut(object) {
            node.locked = true;
        }
        assert!(!session.is_editable(object));
        let center = center_of(&session);
        let response = session
            .dispatch_action(UserAction::Pointer(PointerEvent::Down {
                position: center,
                pressure: 1.0,
            }))
            .expect("dispatch");
        assert!(
            matches!(
                response,
                ToolResponse::Selection(SelectionDelta::ClearObjects)
            ),
            "{response:?}"
        );
    }

    #[test]
    fn headless_render_reports_stats() {
        let mut empty = StudioSession::new("empty");
        let (_, stats) = empty
            .render_headless(&RenderOptions::default())
            .expect("renders");
        assert_eq!(stats.primitives_drawn, 0);

        let mut scene = session_with_rect();
        let (_, stats) = scene
            .render_headless(&RenderOptions::default())
            .expect("renders");
        assert_eq!(stats.primitives_drawn, 1);
    }

    #[test]
    fn hit_test_orders_handle_over_node() {
        let session = session_with_rect();
        let object = session.selection.single().expect("selected");
        let node = session
            .document()
            .scene
            .get_node(object)
            .expect("node")
            .clone();
        let path = node.item_path().expect("path");
        let anchor = path.contours[0].nodes[0].point;
        let hits = session.hit_test(session.view_transform().doc_to_view(anchor));
        assert!(
            matches!(hits.first(), Some(HitTarget::Node { .. })),
            "{hits:?}"
        );
        if let Some(handle) = path.contours[0].nodes[0].handle_out {
            let hits = session.hit_test(session.view_transform().doc_to_view(handle));
            assert!(
                matches!(hits.first(), Some(HitTarget::Handle { .. })),
                "{hits:?}"
            );
        }
    }

    #[test]
    fn smart_delete_preview_warns_instead_of_simplifying() {
        let mut session = session_with_rect();
        let object = session.selection.single().expect("selected");
        let target = NodeId {
            object,
            contour: 0,
            node: 0,
        };
        // Deleting a rectangle corner is a real deformation: the
        // preview shows the candidate curve and an error above the
        // tolerance instead of a silent simplification.
        let ghost = session.smart_delete_preview(target).expect("previews");
        assert!(!ghost.points.is_empty(), "a visible ghost");
        assert!(!ghost.within_tolerance, "error {}", ghost.max_error);
        assert!(ghost.max_error > 1.0, "{}", ghost.max_error);
        // The commit demands an explicit choice, with the same number.
        session.selection.sub.select_node(target);
        let outcome = session
            .smart_delete(petunia_engine::geometry::SmartDeleteMode::PreserveShape)
            .expect("answers");
        let ToolResponse::Failed(message) = outcome else {
            panic!("must refuse, got {outcome:?}");
        };
        assert!(message.contains("Hard Delete"), "{message}");
    }

    #[test]
    fn smart_delete_needs_a_node_selection() {
        let mut session = session_with_rect();
        let response = session
            .smart_delete(petunia_engine::geometry::SmartDeleteMode::HardDelete)
            .expect("dispatch");
        assert!(matches!(response, ToolResponse::Failed(_)), "{response:?}");
    }

    #[test]
    fn smart_delete_commits_one_transaction() {
        let mut session = session_with_rect();
        let object = session.selection.single().expect("selected");
        // Enter Vector Edit and select the first node by stable id.
        session.on_enter();
        assert!(session.contexts().in_vector());
        let node = NodeId {
            object,
            contour: 0,
            node: 0,
        };
        session.selection.sub.select_node(node);
        assert_eq!(session.selection.sub.len(), 1);
        let revision = session.revision();
        let outcome = session
            .smart_delete(petunia_engine::geometry::SmartDeleteMode::HardDelete)
            .expect("delete");
        assert_eq!(session.revision().0, revision.0 + 1, "{outcome:?}");
        // The path now has one node fewer.
        let path = session
            .document()
            .scene
            .get_node(object)
            .and_then(|node| node.item_path())
            .expect("path");
        assert_eq!(path.contours[0].nodes.len(), 3);
    }

    #[test]
    fn nudge_moves_by_document_units_independent_of_zoom() {
        let mut session = session_with_rect();
        let object = session.selection.single().expect("selected");
        session.view_mut().scale = 4.0;
        session
            .dispatch_action(UserAction::Nudge { dx: 1.0, dy: 0.0 })
            .expect("nudge");
        let node = session.document().scene.get_node(object).expect("node");
        assert_eq!(
            node.transform.tx, 1.0,
            "one document unit, not one screen unit"
        );
        // Without a selection the nudge fails loudly instead of doing
        // nothing silently.
        session.selection.clear();
        let response = session
            .dispatch_action(UserAction::Nudge { dx: 1.0, dy: 0.0 })
            .expect("nudge");
        assert!(matches!(response, ToolResponse::Failed(_)), "{response:?}");
    }

    #[test]
    fn escape_prefers_the_focus_layer_then_the_context_stack() {
        let mut session = session_with_rect();
        session.set_focus_entries(vec![
            FocusEntry::new("bar.op", crate::focus::FocusZone::ContextBar, 100, 0),
            FocusEntry::new("canvas", crate::focus::FocusZone::Canvas, 400, 200),
        ]);
        session.focus_mut().focus("bar.op");
        let (focus, _context) = session.on_escape_focus();
        assert_eq!(
            focus,
            crate::focus::FocusOutcome::ReturnedToCanvas {
                entry: "canvas".to_string()
            }
        );
        // Focus is now on the canvas, so the layer stops consuming
        // Escape and the context chain unwinds instead.
        session.on_enter();
        assert!(session.contexts().in_vector());
        let (focus, context) = session.on_escape_focus();
        assert_eq!(focus, crate::focus::FocusOutcome::Unhandled);
        assert!(!session.contexts().in_vector(), "{context:?}");
    }

    #[test]
    fn layers_panel_mirrors_selects_and_toggles() {
        let mut session = session_with_rect();
        let object = session.selection.single().expect("selected");
        session.sync_layers();
        // Page row plus the path row, focus starts on the page.
        assert_eq!(session.layers().rows().len(), 2);
        assert_eq!(
            session.layers().focused().map(|row| row.name.clone()),
            Some("Page 1".to_string())
        );
        // Down to the path, Enter selects it through the panel.
        session.layer_key(LayerKey::Down).expect("down");
        session.layer_key(LayerKey::Enter).expect("enter");
        assert_eq!(session.selection.single(), Some(object));
        assert!(
            session
                .layers()
                .rows()
                .iter()
                .find(|row| row.id == Some(object))
                .expect("row")
                .selected
        );
        // Space hides it in exactly one transaction; undo restores.
        let revision = session.revision();
        session.layer_key(LayerKey::Space).expect("toggle");
        assert_eq!(session.revision().0, revision.0 + 1);
        assert!(
            !session
                .document()
                .scene
                .get_node(object)
                .expect("node")
                .visible
        );
        session.dispatch_action(UserAction::Undo).expect("undo");
        assert!(
            session
                .document()
                .scene
                .get_node(object)
                .expect("node")
                .visible
        );
    }

    #[test]
    fn tooltip_follows_the_live_shortcut_table() {
        let mut session = session_with_rect();
        assert_eq!(
            session.tooltip(ActionId::Undo).as_deref(),
            Some("Desfazer (Ctrl+z)")
        );
        assert!(session
            .shortcuts_mut()
            .rebind(
                ActionId::Undo,
                crate::shortcuts::KeyCombo::key("y").with_ctrl()
            )
            .is_ok());
        assert_eq!(
            session.tooltip(ActionId::Undo).as_deref(),
            Some("Desfazer (Ctrl+y)")
        );
    }

    #[test]
    fn node_drag_moves_nodes_in_one_transaction() {
        let mut session = session_with_rect();
        let object = session.selection.single().expect("selected");
        session.select_tool(ToolKind::NodeEdit);
        // Grab the first anchor at doc (0, 0): view equals doc here.
        session
            .dispatch_action(UserAction::Pointer(PointerEvent::Down {
                position: Point::new(0.0, 0.0),
                pressure: 1.0,
            }))
            .expect("grab");
        session
            .dispatch_action(UserAction::Pointer(PointerEvent::Move {
                position: Point::new(10.0, 20.0),
                pressure: 1.0,
            }))
            .expect("drag");
        let revision = session.revision();
        session
            .dispatch_action(UserAction::Pointer(PointerEvent::Up {
                position: Point::new(10.0, 20.0),
            }))
            .expect("drop");
        assert_eq!(session.revision().0, revision.0 + 1);
        let path = session
            .document()
            .scene
            .get_node(object)
            .and_then(|node| node.item_path())
            .expect("path");
        assert_eq!(path.contours[0].nodes[0].point, Point::new(10.0, 20.0));
        session.dispatch_action(UserAction::Undo).expect("undo");
        let restored = session
            .document()
            .scene
            .get_node(object)
            .and_then(|node| node.item_path())
            .expect("path");
        assert_eq!(restored.contours[0].nodes[0].point, Point::new(0.0, 0.0));
    }

    #[test]
    fn marquee_selects_the_box_by_containment() {
        let mut session = session_with_rect();
        session.selection.clear();
        assert!(session.selection.is_empty());
        // Drag a marquee around the 100x100 box at the origin.
        session
            .dispatch_action(UserAction::Pointer(PointerEvent::Down {
                position: Point::new(-10.0, -10.0),
                pressure: 1.0,
            }))
            .expect("down");
        session
            .dispatch_action(UserAction::Pointer(PointerEvent::Move {
                position: Point::new(110.0, 110.0),
                pressure: 1.0,
            }))
            .expect("drag");
        session
            .dispatch_action(UserAction::Pointer(PointerEvent::Up {
                position: Point::new(110.0, 110.0),
            }))
            .expect("up");
        assert_eq!(session.selection.objects().len(), 1);
    }

    #[test]
    fn pen_creates_a_closed_path_in_one_transaction() {
        let mut session = StudioSession::new("pen");
        session.select_tool(ToolKind::Pen);
        let revision = session.revision();
        for position in [
            Point::new(0.0, 0.0),
            Point::new(60.0, 0.0),
            Point::new(60.0, 40.0),
            // Back on the first anchor closes the path.
            Point::new(0.0, 0.0),
        ] {
            session
                .dispatch_action(UserAction::Pointer(PointerEvent::Down {
                    position,
                    pressure: 1.0,
                }))
                .expect("place");
        }
        assert_eq!(session.revision().0, revision.0 + 1);
        assert_eq!(session.document().scene.len(), 1);
        session.dispatch_action(UserAction::Undo).expect("undo");
        assert_eq!(session.document().scene.len(), 0);
    }

    #[test]
    fn context_label_matches_active_context() {
        let mut session = session_with_rect();
        assert_eq!(session.context_label(), "Scene");
        session.on_enter();
        assert!(session.context_label().starts_with("Vector Edit · node"));
    }

    #[test]
    fn multi_node_bounds_overlay_appears_for_two_plus_nodes() {
        let mut session = session_with_rect();
        let object = session.selection.single().expect("selected");
        session.on_enter();

        // With zero or one node, no multi-node bounds overlay is emitted:
        assert!(session.multi_node_bounds_overlay().is_none());
        session.selection.sub.select_node(NodeId {
            object,
            contour: 0,
            node: 0,
        });
        assert!(session.multi_node_bounds_overlay().is_none());

        // Selecting a second node generates the discrete bounding box (Decision C):
        session.selection.sub.toggle_node(NodeId {
            object,
            contour: 0,
            node: 2,
        });
        assert_eq!(session.selection.sub.nodes().len(), 2);
        let overlay = session
            .multi_node_bounds_overlay()
            .expect("overlay generated");
        let crate::tools::OverlayPrimitive::Rect {
            x,
            y,
            width,
            height,
        } = overlay
        else {
            panic!("expected rect overlay");
        };
        // Node 0 is (0,0), Node 2 is (100,100); view transform is identity:
        assert_eq!((x, y, width, height), (0.0, 0.0, 100.0, 100.0));

        // When toggled off by user preference, overlay is suppressed:
        session.set_show_multi_node_bounds(false);
        assert!(!session.show_multi_node_bounds());
        assert!(session.multi_node_bounds_overlay().is_none());
    }
}
