//! Document commands and shape gestures: prepare complete edits before committing.
use super::*;
use petunia_core::{
    AppearanceKind, BuiltinColorSpace, ColorSource, ColorSpaceRef, ColorValue, Contour, NodeKind,
    Paint, ParentRef, PathNode, ProcessColor, ProcessColorValue, Rgba, SceneNode, Transform2D,
    VectorPath,
};

impl DesktopController {
    pub(super) fn document_command(
        &mut self,
        command: &str,
        payload: &Value,
    ) -> Result<(), DesktopError> {
        match command {
            "shape" => {
                let kind = string(payload, "kind")?;
                let x = optional_number(payload, "x", 80.0)?;
                let y = optional_number(payload, "y", 80.0)?;
                let width = optional_number(payload, "width", 200.0)?;
                let height = optional_number(payload, "height", 140.0)?;
                if width <= 0.0 || height <= 0.0 {
                    return invalid("Largura e altura devem ser positivas");
                }
                self.insert_shape(kind, Rect::new(x, y, width, height))
            }
            "selectLayer" => {
                self.session.set_selection(vec![object_id(payload)?])?;
                Ok(())
            }
            "selectNode" => {
                let node = node_target(payload)?;
                self.session
                    .select_node(node, boolean_or(payload, "additive", false)?)?;
                self.status = "Nó selecionado; use as coordenadas ou as setas para mover".into();
                Ok(())
            }
            "moveNode" => self.move_node(payload),
            "smartDelete" => {
                let mode = match payload
                    .get("mode")
                    .and_then(Value::as_str)
                    .unwrap_or("preserve")
                {
                    "preserve" => petunia_engine::geometry::SmartDeleteMode::PreserveShape,
                    "hard" => petunia_engine::geometry::SmartDeleteMode::HardDelete,
                    _ => return invalid("Modo de exclusão de nó desconhecido"),
                };
                let response = self.session.smart_delete(mode)?;
                self.response(response)?;
                let ids = self.session.selection().objects().to_vec();
                self.session.set_selection(ids)?;
                Ok(())
            }
            "selectAll" => {
                self.session.sync_layers();
                let ids = self
                    .session
                    .layers()
                    .rows()
                    .iter()
                    .filter_map(|row| row.id)
                    .filter(|id| self.session.is_editable(*id))
                    .collect();
                self.session.set_selection(ids)?;
                Ok(())
            }
            "delete" => {
                if !self.session.selection().sub.nodes().is_empty() {
                    return self
                        .document_command("smartDelete", &serde_json::json!({"mode":"preserve"}));
                }
                let roots = self.editable_selection()?;
                self.commit(
                    vec![DocumentOp::RemoveObjects { roots }],
                    HistoryDescription::DeleteObjects,
                )?;
                self.session.set_selection(Vec::new())?;
                self.status = "Seleção removida".into();
                Ok(())
            }
            "duplicate" => {
                self.editable_selection()?;
                let clipboard = self.session.copy_selection()?;
                self.session.paste(&clipboard)?;
                self.status = "Seleção duplicada".into();
                Ok(())
            }
            "visibility" | "locked" => {
                let id = object_id(payload)?;
                let node = self
                    .session
                    .document()
                    .scene
                    .get_node(id)
                    .ok_or_else(|| DesktopError::Invalid("Objeto inexistente".into()))?;
                let operation = if command == "visibility" {
                    let visible = boolean_or(payload, "visible", !node.visible)?;
                    if visible == node.visible {
                        return Ok(());
                    }
                    DocumentOp::SetVisibility {
                        object: id,
                        visible,
                    }
                } else {
                    let locked = boolean_or(payload, "locked", !node.locked)?;
                    if locked == node.locked {
                        return Ok(());
                    }
                    DocumentOp::SetLocked { object: id, locked }
                };
                self.commit(vec![operation], HistoryDescription::EditObjects)?;
                let remaining = self
                    .session
                    .selection()
                    .objects()
                    .iter()
                    .copied()
                    .filter(|id| self.session.is_editable(*id))
                    .collect();
                self.session.set_selection(remaining)?;
                Ok(())
            }
            "setFill" => self.set_fill(string(payload, "color")?),
            "transform" => self.transform(payload),
            _ => invalid("Comando documental indisponível"),
        }
    }

    fn editable_selection(&self) -> Result<Vec<ObjectId>, DesktopError> {
        let ids = self.session.selection().objects().to_vec();
        if ids.is_empty() {
            return invalid("Selecione um objeto primeiro");
        }
        if ids.iter().any(|id| !self.session.is_editable(*id)) {
            return invalid("A seleção contém objeto travado ou oculto");
        }
        Ok(ids)
    }

    pub(super) fn nudge_nodes(&mut self, dx: f64, dy: f64) -> Result<(), DesktopError> {
        let nodes = self.session.selection().sub.nodes().to_vec();
        let mut paths = std::collections::BTreeMap::new();
        for target in nodes {
            if !self.session.is_editable(target.object) {
                return invalid("Nó oculto ou bloqueado");
            }
            let scene = &self.session.document().scene;
            if let std::collections::btree_map::Entry::Vacant(entry) = paths.entry(target.object) {
                entry.insert(
                    scene
                        .get_node(target.object)
                        .and_then(SceneNode::item_path)
                        .cloned()
                        .ok_or_else(|| DesktopError::Invalid("Caminho ausente".into()))?,
                );
            }
            let world = scene
                .world_transform(target.object)
                .ok_or_else(|| DesktopError::Invalid("Transformação inválida".into()))?;
            let inverse = world
                .inverse()
                .ok_or_else(|| DesktopError::Invalid("Transformação singular".into()))?;
            let path = paths
                .get_mut(&target.object)
                .ok_or_else(|| DesktopError::Invalid("Caminho ausente".into()))?;
            let node = path
                .contours
                .get_mut(target.contour as usize)
                .and_then(|contour| contour.nodes.get_mut(target.node as usize))
                .ok_or_else(|| DesktopError::Invalid("Nó ausente".into()))?;
            let before = world.transform_point(node.point);
            let after = inverse.transform_point(Point::new(before.x + dx, before.y + dy));
            let local_dx = after.x - node.point.x;
            let local_dy = after.y - node.point.y;
            node.point = after;
            for handle in [&mut node.handle_in, &mut node.handle_out]
                .into_iter()
                .flatten()
            {
                handle.x += local_dx;
                handle.y += local_dy;
            }
        }
        let operations = paths
            .into_iter()
            .map(|(object, path)| DocumentOp::ReplacePath { object, path })
            .collect();
        self.commit(operations, HistoryDescription::EditObjects)?;
        self.status = "Nós movidos em unidades do documento".into();
        Ok(())
    }

    fn move_node(&mut self, payload: &Value) -> Result<(), DesktopError> {
        let target = node_target(payload)?;
        if !self.session.is_editable(target.object) {
            return invalid("O nó está ausente, oculto ou bloqueado");
        }
        let world = self
            .session
            .document()
            .scene
            .world_transform(target.object)
            .ok_or_else(|| DesktopError::Invalid("Transformação do objeto inválida".into()))?;
        let inverse = world
            .inverse()
            .ok_or_else(|| DesktopError::Invalid("Transformação do objeto singular".into()))?;
        let mut path = self
            .session
            .document()
            .scene
            .get_node(target.object)
            .and_then(SceneNode::item_path)
            .cloned()
            .ok_or_else(|| DesktopError::Invalid("Objeto não possui caminho editável".into()))?;
        let node = path
            .contours
            .get_mut(target.contour as usize)
            .and_then(|contour| contour.nodes.get_mut(target.node as usize))
            .ok_or_else(|| DesktopError::Invalid("Nó inexistente".into()))?;
        let before = world.transform_point(node.point);
        let x = optional_number(payload, "x", before.x)?;
        let y = optional_number(payload, "y", before.y)?;
        if x == before.x && y == before.y {
            return Ok(());
        }
        let next = inverse.transform_point(Point::new(x, y));
        let delta = Point::new(next.x - node.point.x, next.y - node.point.y);
        node.point = next;
        for handle in [&mut node.handle_in, &mut node.handle_out]
            .into_iter()
            .flatten()
        {
            handle.x += delta.x;
            handle.y += delta.y;
        }
        self.commit(
            vec![DocumentOp::ReplacePath {
                object: target.object,
                path,
            }],
            HistoryDescription::EditObjects,
        )?;
        self.session.select_node(target, false)?;
        self.status = "Coordenadas do nó atualizadas".into();
        Ok(())
    }

    pub(super) fn selection_bounds(&self) -> Option<Rect> {
        self.session
            .selection()
            .objects()
            .iter()
            .filter_map(|id| self.session.object_bounds(*id))
            .reduce(|a, b| a.union(b))
    }

    fn set_fill(&mut self, color: &str) -> Result<(), DesktopError> {
        let rgba = parse_color(color)?;
        let paint = Paint::Solid(ColorSource::Value(ColorValue::Process(ProcessColor {
            value: ProcessColorValue::Rgb(rgba),
            space: ColorSpaceRef::Builtin(BuiltinColorSpace::Srgb),
        })));
        let mut operations = Vec::new();
        for id in self.editable_selection()? {
            let previous = self
                .session
                .document()
                .scene
                .get_node(id)
                .and_then(SceneNode::item_appearance)
                .cloned()
                .ok_or_else(|| {
                    DesktopError::Invalid("Objeto não possui aparência editável".into())
                })?;
            let mut appearance = previous.clone();
            if let Some(item) = appearance
                .items
                .iter_mut()
                .find(|item| matches!(item.kind, AppearanceKind::Fill(_)))
            {
                item.kind = AppearanceKind::Fill(paint.clone());
                item.enabled = true;
            } else {
                appearance.items.push(
                    petunia_core::AppearanceItem::new(
                        true,
                        1.0,
                        petunia_core::BlendMode::Normal,
                        AppearanceKind::Fill(paint.clone()),
                    )
                    .map_err(|error| DesktopError::Invalid(error.to_string()))?,
                );
            }
            if appearance != previous {
                operations.push(DocumentOp::SetAppearance {
                    object: id,
                    appearance,
                });
            }
        }
        self.commit(operations, HistoryDescription::SetFill)?;
        self.status = "Preenchimento atualizado".into();
        Ok(())
    }

    fn transform(&mut self, payload: &Value) -> Result<(), DesktopError> {
        let ids = self.editable_selection()?;
        let bounds = self
            .selection_bounds()
            .ok_or_else(|| DesktopError::Invalid("Seleção sem limites geométricos".into()))?;
        let x = optional_number(payload, "x", bounds.x)?;
        let y = optional_number(payload, "y", bounds.y)?;
        let width = optional_number(payload, "width", bounds.width)?;
        let height = optional_number(payload, "height", bounds.height)?;
        if width <= 0.0 || height <= 0.0 || bounds.width <= 0.0 || bounds.height <= 0.0 {
            return invalid("Dimensões devem ser positivas; selecione geometria com área");
        }
        if x == bounds.x && y == bounds.y && width == bounds.width && height == bounds.height {
            return Ok(());
        }
        let world_delta = Transform2D::translation(x, y)
            .concat(Transform2D::scale(
                width / bounds.width,
                height / bounds.height,
            ))
            .concat(Transform2D::translation(-bounds.x, -bounds.y));
        let scene = &self.session.document().scene;
        let mut operations = Vec::new();
        for id in &ids {
            // Transform selected roots only: children already follow their parents.
            if scene
                .ancestors(*id)
                .iter()
                .any(|parent| ids.contains(parent))
            {
                continue;
            }
            let node = scene
                .get_node(*id)
                .ok_or_else(|| DesktopError::Invalid("Objeto inexistente".into()))?;
            let parent_world = match node.parent {
                ParentRef::Page(_) => Transform2D::IDENTITY,
                ParentRef::Object(parent) => scene.world_transform(parent).ok_or_else(|| {
                    DesktopError::Invalid("Transformação ancestral inválida".into())
                })?,
            };
            let inverse = parent_world
                .inverse()
                .ok_or_else(|| DesktopError::Invalid("Transformação ancestral singular".into()))?;
            let world = scene
                .world_transform(*id)
                .ok_or_else(|| DesktopError::Invalid("Transformação inválida".into()))?;
            operations.push(DocumentOp::SetTransform {
                object: *id,
                transform: inverse.concat(world_delta).concat(world),
            });
        }
        self.commit(operations, HistoryDescription::SetTransform)?;
        self.status = "Transformação aplicada".into();
        Ok(())
    }

    fn insert_shape(&mut self, kind: &str, bounds: Rect) -> Result<(), DesktopError> {
        let path = match kind {
            "rectangle" => VectorPath::rect(bounds.x, bounds.y, bounds.width, bounds.height),
            "ellipse" => ellipse(bounds),
            _ => return invalid("Forma desconhecida"),
        };
        let name = if kind == "rectangle" {
            "Retângulo"
        } else {
            "Elipse"
        };
        let node = SceneNode::new_path(name, path, ParentRef::Page(self.session.active_page()));
        let id = node.id;
        let index = self
            .session
            .document()
            .scene
            .page_roots(self.session.active_page())
            .unwrap_or_default()
            .len();
        self.commit(
            vec![DocumentOp::InsertRoot {
                index,
                node: Box::new(node),
            }],
            HistoryDescription::InsertObjects,
        )?;
        self.session.set_selection(vec![id])?;
        self.status = format!("{name} criado");
        Ok(())
    }

    pub(super) fn shape_pointer(
        &mut self,
        phase: &str,
        point: Point,
        shift: bool,
    ) -> Result<(), DesktopError> {
        match phase {
            "down" => {
                if self.shape_origin.is_some() {
                    return Err(DesktopError::PendingInteraction);
                }
                self.shape_origin = Some(point);
                self.status = "Arraste para criar; Escape cancela".into();
            }
            "move" | "up" => {
                let Some(origin) = self.shape_origin else {
                    return Ok(());
                };
                let mut dx = point.x - origin.x;
                let mut dy = point.y - origin.y;
                if shift {
                    let side = dx.abs().max(dy.abs());
                    dx = side * if dx < 0.0 { -1.0 } else { 1.0 };
                    dy = side * if dy < 0.0 { -1.0 } else { 1.0 };
                }
                let view = self.session.view();
                let bounds = Rect::new(
                    origin.x.min(origin.x + dx),
                    origin.y.min(origin.y + dy),
                    dx.abs(),
                    dy.abs(),
                );
                self.overlays = vec![
                    serde_json::json!({"type":"marquee","x":bounds.x,"y":bounds.y,"width":bounds.width,"height":bounds.height,"shape":tool_id(self.session.active_tool())}),
                ];
                if phase == "up" {
                    self.shape_origin = None;
                    self.overlays.clear();
                    if bounds.width < 1.0 || bounds.height < 1.0 {
                        self.status = "Forma cancelada: arraste para definir tamanho".into();
                        return Ok(());
                    }
                    let document_bounds = Rect::new(
                        (bounds.x - view.pan_x) / view.scale,
                        (bounds.y - view.pan_y) / view.scale,
                        bounds.width / view.scale,
                        bounds.height / view.scale,
                    );
                    self.insert_shape(tool_id(self.session.active_tool()), document_bounds)?;
                }
            }
            _ => return invalid("Fase inválida"),
        }
        Ok(())
    }
}

pub(super) fn node_target(payload: &Value) -> Result<petunia_ui::NodeId, DesktopError> {
    let object = serde_json::from_value(
        payload
            .get("object")
            .cloned()
            .ok_or_else(|| DesktopError::Invalid("Objeto obrigatório".into()))?,
    )?;
    let index = |key: &str| -> Result<u32, DesktopError> {
        payload
            .get(key)
            .and_then(Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| DesktopError::Invalid(format!("Índice de nó inválido: {key}")))
    };
    Ok(petunia_ui::NodeId {
        object,
        contour: index("contour")?,
        node: index("node")?,
    })
}

fn ellipse(bounds: Rect) -> VectorPath {
    let cx = bounds.x + bounds.width / 2.0;
    let cy = bounds.y + bounds.height / 2.0;
    let rx = bounds.width / 2.0;
    let ry = bounds.height / 2.0;
    let k = 0.552_284_749_830_793_6;
    let points = [
        (
            Point::new(cx + rx, cy),
            Point::new(cx + rx, cy - k * ry),
            Point::new(cx + rx, cy + k * ry),
        ),
        (
            Point::new(cx, cy + ry),
            Point::new(cx + k * rx, cy + ry),
            Point::new(cx - k * rx, cy + ry),
        ),
        (
            Point::new(cx - rx, cy),
            Point::new(cx - rx, cy + k * ry),
            Point::new(cx - rx, cy - k * ry),
        ),
        (
            Point::new(cx, cy - ry),
            Point::new(cx - k * rx, cy - ry),
            Point::new(cx + k * rx, cy - ry),
        ),
    ];
    let mut contour = Contour::new(true);
    for (point, incoming, outgoing) in points {
        contour.push_node(PathNode::with_handles(
            point,
            Some(incoming),
            Some(outgoing),
            NodeKind::Smooth,
        ));
    }
    let mut path = VectorPath::new();
    path.push_contour(contour);
    path
}

pub(super) fn parse_color(text: &str) -> Result<Rgba, DesktopError> {
    let hex = text
        .strip_prefix('#')
        .ok_or_else(|| DesktopError::Invalid("Cor deve usar #RRGGBB ou #AARRGGBB".into()))?;
    if !matches!(hex.len(), 6 | 8) || !hex.is_ascii() {
        return invalid("Cor deve usar #RRGGBB ou #AARRGGBB");
    }
    let mut values = [255u8; 4];
    for (i, slot) in values.iter_mut().enumerate().take(hex.len() / 2) {
        *slot = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
            .map_err(|_| DesktopError::Invalid("Canal de cor inválido".into()))?;
    }
    let (r, g, b, alpha) = if hex.len() == 8 {
        (values[1], values[2], values[3], values[0])
    } else {
        (values[0], values[1], values[2], 255)
    };
    Ok(Rgba {
        r: f32::from(r) / 255.0,
        g: f32::from(g) / 255.0,
        b: f32::from(b) / 255.0,
        alpha: f32::from(alpha) / 255.0,
    })
}

pub(super) fn fill_hex(node: &SceneNode) -> Option<String> {
    let appearance = node.item_appearance()?;
    appearance.items.iter().find_map(|item| {
        if !item.enabled {
            return None;
        }
        if let AppearanceKind::Fill(Paint::Solid(ColorSource::Value(ColorValue::Process(color)))) =
            &item.kind
        {
            let rgba = color.as_rgb()?;
            if color.space != ColorSpaceRef::Builtin(BuiltinColorSpace::Srgb) {
                return None;
            }
            let rgb = format!(
                "{:02X}{:02X}{:02X}",
                (rgba.r.clamp(0.0, 1.0) * 255.0).round() as u8,
                (rgba.g.clamp(0.0, 1.0) * 255.0).round() as u8,
                (rgba.b.clamp(0.0, 1.0) * 255.0).round() as u8
            );
            return Some(if rgba.alpha >= 1.0 {
                format!("#{rgb}")
            } else {
                format!("#{:02X}{rgb}", (rgba.alpha * 255.0).round() as u8)
            });
        }
        None
    })
}
