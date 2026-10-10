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
use crate::numeric::NumericField;
use crate::shortcuts::ActionId;
use crate::shortcuts::ShortcutTable;
use crate::tools::{
    distance, HitTarget, ItemKind, NodeTool, PenTool, PointerSample, SelectTool, SelectionDelta,
    ToolController, ToolResponse, ToolServices, ToolSession, ViewTransform,
};
use crate::tooltips::tooltip_for;
use crate::workspace::{ViewState, WorkspaceState};
use petunia_core::{
    Document, FillRule, ObjectId, ParentRef, Point, SceneItem, SceneNode, Size2, VectorPath,
};
use petunia_engine::compile;
use petunia_engine::geometry::{delete_node, SmartDeleteMode, SmartDeleteOutcome};
use petunia_engine::EngineError;
use petunia_engine::History;
use petunia_engine::{prepare_transaction, TransactionRequest};
use petunia_engine::{CommandId, DocumentOp, DocumentRevision, HistoryDescription};
use petunia_render::RenderOptions;
use petunia_render::SoftwareRenderer;
use petunia_render_model::RenderStats;

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
    captured: bool,
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
        Self {
            document: Document::new(name),
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
            captured: false,
        }
    }

    /// Current authorial revision.
    #[must_use]
    pub fn revision(&self) -> DocumentRevision {
        self.history.current_revision()
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
        if self.captured {
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

    /// Whether an object may be edited: exists, visible and unlocked.
    #[must_use]
    pub fn is_editable(&self, id: ObjectId) -> bool {
        self.document
            .scene
            .get_node(id)
            .is_some_and(|node| node.visible && !node.locked)
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
                Ok(ToolResponse::Idle)
            }
            UserAction::Redo => {
                self.history.redo(&mut self.document)?;
                Ok(ToolResponse::Idle)
            }
            UserAction::SmartDelete(mode) => self.smart_delete(mode),
            UserAction::Nudge { dx, dy } => self.nudge(dx, dy),
            // Hover never mutates: a tooltip is Session State at most.
            UserAction::HoverTooltip(_) => Ok(ToolResponse::Idle),
            UserAction::Pointer(pointer) => self.handle_pointer(pointer),
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
        self.commit_request(TransactionRequest {
            command_id: CommandId::new_v4(),
            operations,
            merge_key: None,
        })?;
        let worst = deviations
            .iter()
            .fold(0.0_f64, |worst, error| worst.max(*error));
        Ok(ToolResponse::Status(format!(
            "Smart Delete aplicado, erro máximo {worst:.3}"
        )))
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
        let (sample, is_end) = match pointer {
            PointerEvent::Down { position, .. } => (PointerSample::new(position), false),
            PointerEvent::Move { position, .. } => (PointerSample::new(position), false),
            PointerEvent::Up { position } => (PointerSample::new(position), true),
        };
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
                self.commit_request(TransactionRequest {
                    command_id: CommandId::new_v4(),
                    operations: operations.clone(),
                    merge_key: None,
                })?;
            }
            _ => {}
        }
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
        let (snapshot, warnings) =
            compile::compile_document(&self.document, self.revision(), options.quality);
        for warning in &warnings {
            eprintln!(
                "[render] primitiva degradada {}: {}",
                warning.source, warning.message
            );
        }
        let width = self.view.viewport.width.max(1.0) as u32;
        let height = self.view.viewport.height.max(1.0) as u32;
        let frame = compile::headless_frame(snapshot, width, height, self.view.scale);
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
        let document_point = self.view_transform().view_to_doc(view_point);
        let tolerance = self.view.document_tolerance(8.0).unwrap_or(8.0);

        let mut targets: Vec<HitTarget> = Vec::new();
        let mut fills: Vec<HitTarget> = Vec::new();

        for id in self.document.scene.root_order() {
            let Some(node) = self.document.scene.get_node(*id) else {
                continue;
            };
            if !node.visible {
                continue;
            }
            let SceneItem::Path(object) = &node.item else {
                continue;
            };
            let path = &object.path;
            let Some(local) = self.to_local(node, document_point) else {
                continue;
            };
            let (node_hit, handle_hit) = node_hit(path, local, tolerance);
            if let Some((node, handle)) = handle_hit {
                targets.push(HitTarget::Handle {
                    object: *id,
                    contour: node.0,
                    node: node.1,
                    handle,
                });
                continue;
            }
            if let Some((contour, node)) = node_hit {
                targets.push(HitTarget::Node {
                    object: *id,
                    contour,
                    node,
                });
                continue;
            }
            if path_contains(path, local) {
                fills.push(HitTarget::Fill { object: *id });
            }
        }

        targets.extend(fills);
        targets
    }

    fn to_local(&self, node: &SceneNode, document_point: Point) -> Option<Point> {
        let local = node.transform.inverse()?.transform_point(document_point);
        (local.x.is_finite() && local.y.is_finite()).then_some(local)
    }

    /// Geometric bounds of one object from its path anchors, unioned
    /// through groups. `None` for items without computable bounds.
    fn object_bounds(&self, id: ObjectId) -> Option<petunia_core::Rect> {
        let node = self.document.scene.get_node(id)?;
        match &node.item {
            SceneItem::Path(object) => {
                let mut bounds: Option<petunia_core::Rect> = None;
                for contour in &object.path.contours {
                    for anchor in contour.nodes.iter().map(|node| node.point) {
                        let slot = petunia_core::Rect::new(anchor.x, anchor.y, 0.0, 0.0);
                        bounds = Some(match bounds {
                            Some(existing) => existing.union(slot),
                            None => slot,
                        });
                    }
                }
                bounds
            }
            SceneItem::Group(children) => {
                let mut bounds: Option<petunia_core::Rect> = None;
                for child in children {
                    if let Some(child_bounds) = self.object_bounds(*child) {
                        bounds = Some(match bounds {
                            Some(existing) => existing.union(child_bounds),
                            None => child_bounds,
                        });
                    }
                }
                bounds
            }
            _ => None,
        }
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
            offset_x: self.view.pan_x,
            offset_y: self.view.pan_y,
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
        let page = self.document.scene.default_page();
        let node = SceneNode::new_path(name, path, ParentRef::Page(page));
        let request = TransactionRequest {
            command_id: CommandId::new_v4(),
            operations: vec![DocumentOp::InsertRoot {
                index: self.document.scene.len(),
                node: Box::new(node),
            }],
            merge_key: None,
        };
        self.commit_request(request)
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
        self.commit_request(TransactionRequest {
            command_id: CommandId::new_v4(),
            operations,
            merge_key: None,
        })?;
        Ok(true)
    }

    /// Commit one request atomically through the session.
    pub fn commit_request(
        &mut self,
        request: TransactionRequest,
    ) -> std::result::Result<(), EngineError> {
        if request.operations.is_empty() {
            return Ok(());
        }
        let prepared = prepare_transaction(&self.document, request, self.revision())
            .map_err(|error| EngineError::Execution(error.to_string()))?;
        // The session returns `EngineError`, so the new revision is
        // redundant to the caller; the caller can read `revision()`.
        self.history
            .commit(
                &mut self.document,
                prepared,
                HistoryDescription::MoveObjects,
            )
            .map(|_| ())
    }
}

impl ToolServices for StudioSession {
    fn hit_test(&self, view_point: Point) -> Vec<HitTarget> {
        StudioSession::hit_test(self, view_point)
    }

    fn snap(&self, view_point: Point) -> (Point, Option<String>) {
        // Guides, grid and bounds providers still have no persistent
        // store in Core, so the point passes through uncorrected.
        (view_point, None)
    }

    fn marquee_select(&self, min: Point, max: Point) -> Vec<ObjectId> {
        let mut selected = Vec::new();
        for id in self.document.scene.root_order() {
            let Some(bounds) = self.object_bounds(*id) else {
                continue;
            };
            let lower = bounds.min();
            let upper = bounds.max();
            let inside =
                lower.x >= min.x && lower.y >= min.y && upper.x <= max.x && upper.y <= max.y;
            if inside {
                selected.push(*id);
            }
        }
        selected
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
        self.document.scene.default_page()
    }

    fn root_count(&self) -> usize {
        self.document.scene.len()
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

fn path_contains(path: &VectorPath, local: Point) -> bool {
    for contour in &path.contours {
        if !contour.closed || contour.nodes.len() < 3 {
            continue;
        }
        let ring: Vec<Point> = contour.nodes.iter().map(|node| node.point).collect();
        if point_in_polygon(local, &ring, path.fill_rule) {
            return true;
        }
    }
    false
}

fn point_in_polygon(point: Point, ring: &[Point], rule: FillRule) -> bool {
    let mut winding = 0i32;
    let count = ring.len();
    for index in 0..count {
        let a = ring[index];
        let b = ring[(index + 1) % count];
        let cross = (b.x - a.x) * (point.y - a.y) - (point.x - a.x) * (b.y - a.y);
        if a.y <= point.y {
            if b.y > point.y && cross > 0.0 {
                winding += 1;
            }
        } else if b.y <= point.y && cross < 0.0 {
            winding -= 1;
        }
    }
    match rule {
        FillRule::NonZero => winding != 0,
        FillRule::EvenOdd => winding % 2 != 0,
    }
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
        assert!(matches!(response, ToolResponse::Failed(_)), "{response:?}");
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
}
