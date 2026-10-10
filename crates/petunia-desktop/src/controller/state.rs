//! Presentation DTOs. No Qt types or document serialization cross this edge.
use super::*;
use petunia_ui::{ActionId, OverlayPrimitive};
use serde_json::json;

impl DesktopController {
    pub fn state(&mut self) -> Value {
        self.session.sync_layers();
        let layers:Vec<_>=self.session.layers().rows().iter().filter(|row|row.id.is_some()).map(|row|{
            let parent=row.id.and_then(|id|self.session.document().scene.parent_of(id)).and_then(|parent|if let petunia_core::ParentRef::Object(id)=parent{Some(id)}else{None});
            json!({"id":row.id,"name":row.name,"depth":row.depth.saturating_sub(1),"visible":row.visible,"locked":row.locked,"selected":row.selected,"label":row.label(),"parent":parent,"hasChildren":row.child_count>0,"group":row.kind==petunia_ui::LayerKind::Group})
        }).collect();
        let history:Vec<_>=self.session.history_rows().iter().map(|(description,revision,applied)|json!({"label":history_label(*description),"revision":revision.0,"applied":applied})).collect();
        let selection = self.selection_state();
        let view = self.session.view();
        let (width, height) = self.page_size();
        let mut preferences = serde_json::to_value(&self.preferences).unwrap_or_else(|_| json!({}));
        preferences["panels"] = json!(self
            .session
            .workspace()
            .panels()
            .iter()
            .map(|panel| json!({"id":panel.id,"open":panel.open}))
            .collect::<Vec<_>>());
        json!({
            "title":self.session.document().metadata.title,
            "path":self.session.destination_path().map(|path|path.to_string_lossy().into_owned()),
            "dirty":self.session.is_dirty(),"revision":self.session.revision().0,
            "tool":tool_id(self.session.active_tool()),"context":self.session.context_label(),
            "status":self.status,"error":self.error,"busy":false,
            "canUndo":self.session.can_undo(),"canRedo":self.session.can_redo(),
            "zoom":view.scale*100.0,"viewport":{"scale":view.scale,"panX":view.pan_x,"panY":view.pan_y},
            "gestureActive":self.shape_origin.is_some()||self.session.interaction_active(),
            "selectionCount":self.session.selection().len(),"selection":selection,
            "layers":layers,"history":history,"preferences":preferences,"nodes":self.node_rows(),
            "actions":self.actions(),"overlays":self.current_overlays(),
            "documentWidth":width,"documentHeight":height,
        })
    }

    fn selection_state(&self) -> Value {
        let bounds = self
            .selection_bounds()
            .unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0));
        let node = self
            .session
            .selection()
            .single()
            .and_then(|id| self.session.document().scene.get_node(id));
        let fill = node.and_then(super::document::fill_hex).unwrap_or_default();
        let kind = node
            .map(|node| match &node.item {
                petunia_core::SceneItem::Path(_) => "path",
                petunia_core::SceneItem::Shape(_) => "shape",
                petunia_core::SceneItem::Group(_) => "group",
                petunia_core::SceneItem::Text(_) => "text",
                _ => "other",
            })
            .unwrap_or("multiple");
        let editable = !self.session.selection().is_empty()
            && self
                .session
                .selection()
                .objects()
                .iter()
                .all(|id| self.session.is_editable(*id));
        json!({"x":bounds.x,"y":bounds.y,"width":bounds.width,"height":bounds.height,"fill":fill,"kind":kind,"editable":editable})
    }

    fn node_rows(&self) -> Vec<Value> {
        let mut rows = Vec::new();
        for object in self.session.selection().objects() {
            let Some(scene_node) = self.session.document().scene.get_node(*object) else {
                continue;
            };
            let Some(path) = scene_node.item_path() else {
                continue;
            };
            let Some(world) = self.session.document().scene.world_transform(*object) else {
                continue;
            };
            for (contour_index, contour) in path.contours.iter().enumerate() {
                for (node_index, node) in contour.nodes.iter().enumerate() {
                    let target = petunia_ui::NodeId {
                        object: *object,
                        contour: contour_index as u32,
                        node: node_index as u32,
                    };
                    let point = world.transform_point(node.point);
                    let kind = match node.kind {
                        petunia_core::NodeKind::Cusp => "cusp",
                        petunia_core::NodeKind::Smooth => "smooth",
                        petunia_core::NodeKind::Symmetric => "symmetric",
                    };
                    rows.push(json!({"object":object,"contour":contour_index,"node":node_index,"id":node.id,"x":point.x,"y":point.y,"kind":kind,"selected":self.session.selection().sub.nodes().contains(&target),"editable":self.session.is_editable(*object),"label":format!("{}, contorno {}, nó {} ({kind}), X {:.2}, Y {:.2}",scene_node.name,contour_index+1,node_index+1,point.x,point.y)}));
                }
            }
        }
        rows
    }

    fn actions(&self) -> Vec<Value> {
        let idle = self.require_idle().is_ok();
        let has_selection = !self.session.selection().is_empty();
        let mut rows = Vec::new();
        let entries = [
            ("document.new", "Novo documento", None, true, ""),
            ("document.open", "Abrir documento", None, true, ""),
            ("document.save", "Salvar", None, true, ""),
            ("document.saveAs", "Salvar como", None, true, ""),
            ("document.exportPng", "Exportar PNG", None, true, ""),
            (
                "tool.select",
                "Seleção",
                Some(ActionId::SelectTool),
                true,
                "",
            ),
            (
                "tool.node",
                "Editar nós",
                Some(ActionId::NodeTool),
                true,
                "",
            ),
            ("tool.pen", "Caneta", Some(ActionId::PenTool), true, ""),
            ("tool.rectangle", "Retângulo", None, true, ""),
            ("tool.ellipse", "Elipse", None, true, ""),
            ("tool.zoom", "Zoom", None, true, ""),
            (
                "tool.brush",
                "Pincel",
                Some(ActionId::BrushTool),
                false,
                "Ferramenta de pintura ainda não conectada à janela",
            ),
            (
                "tool.eraser",
                "Borracha",
                Some(ActionId::EraserTool),
                false,
                "Ferramenta de pintura ainda não conectada à janela",
            ),
            (
                "tool.text",
                "Texto",
                None,
                false,
                "Ferramenta de texto ainda não conectada à janela",
            ),
            (
                "tool.pan",
                "Deslocar canvas",
                Some(ActionId::PanTool),
                false,
                "Use Espaço e arraste o canvas",
            ),
            (
                "edit.undo",
                "Desfazer",
                Some(ActionId::Undo),
                self.session.can_undo(),
                "Nenhuma alteração para desfazer",
            ),
            (
                "edit.redo",
                "Refazer",
                Some(ActionId::Redo),
                self.session.can_redo(),
                "Nenhuma alteração para refazer",
            ),
            (
                "edit.select_all",
                "Selecionar tudo",
                Some(ActionId::SelectAll),
                true,
                "",
            ),
            (
                "edit.delete",
                "Excluir seleção",
                None,
                has_selection,
                "Selecione um objeto primeiro",
            ),
            (
                "edit.duplicate",
                "Duplicar seleção",
                None,
                has_selection,
                "Selecione um objeto primeiro",
            ),
            (
                "edit.hard_delete_nodes",
                "Excluir nós diretamente",
                None,
                !self.session.selection().sub.nodes().is_empty(),
                "Selecione um nó primeiro",
            ),
            (
                "edit.group",
                "Agrupar",
                Some(ActionId::Group),
                false,
                "Agrupamento ainda não conectado à janela",
            ),
            (
                "edit.ungroup",
                "Desagrupar",
                Some(ActionId::Ungroup),
                false,
                "Agrupamento ainda não conectado à janela",
            ),
            (
                "edit.bring_forward",
                "Mover para frente",
                Some(ActionId::BringForward),
                false,
                "Reordenação ainda não conectada à janela",
            ),
            (
                "edit.send_backward",
                "Mover para trás",
                Some(ActionId::SendBackward),
                false,
                "Reordenação ainda não conectada à janela",
            ),
            (
                "edit.confirm",
                "Confirmar",
                Some(ActionId::Confirm),
                true,
                "",
            ),
            ("edit.cancel", "Cancelar", Some(ActionId::Cancel), true, ""),
            (
                "view.pan",
                "Deslocar visualização",
                Some(ActionId::PanTool),
                false,
                "Use Espaço e arraste o canvas para deslocar a visualização",
            ),
            ("view.fit", "Ajustar página", None, true, ""),
            ("view.zoom_in", "Aproximar", None, true, ""),
            ("view.zoom_out", "Afastar", None, true, ""),
            (
                "nudge.left",
                "Mover à esquerda",
                Some(ActionId::NudgeLeft),
                has_selection,
                "Selecione um objeto primeiro",
            ),
            (
                "nudge.right",
                "Mover à direita",
                Some(ActionId::NudgeRight),
                has_selection,
                "Selecione um objeto primeiro",
            ),
            (
                "nudge.up",
                "Mover acima",
                Some(ActionId::NudgeUp),
                has_selection,
                "Selecione um objeto primeiro",
            ),
            (
                "nudge.down",
                "Mover abaixo",
                Some(ActionId::NudgeDown),
                has_selection,
                "Selecione um objeto primeiro",
            ),
        ];
        for (id, label, action, available, reason) in entries {
            let session_only = matches!(id, "edit.confirm" | "edit.cancel");
            let enabled = available && (idle || session_only);
            let reason = if !idle && !session_only {
                "Confirme ou cancele a operação atual"
            } else if available {
                ""
            } else {
                reason
            };
            let shortcut = action
                .and_then(|action| self.session.shortcuts().combo_for(action))
                .map(|combo| combo.display())
                .unwrap_or_default();
            rows.push(json!({"id":id,"label":label,"shortcut":shortcut,"enabled":enabled,"reason":reason,"rebindable":action.is_some()}));
        }
        for (id, label, action) in [
            (
                "workspace.palette",
                "Paleta de comandos",
                ActionId::CommandPalette,
            ),
            (
                "workspace.personas",
                "Gerenciar personas",
                ActionId::ManagePersonas,
            ),
            (
                "workspace.preferences",
                "Preferências",
                ActionId::Preferences,
            ),
            (
                "workspace.shortcuts",
                "Editor de atalhos",
                ActionId::ShortcutEditor,
            ),
            (
                "workspace.focus_next",
                "Próxima região",
                ActionId::FocusNext,
            ),
            (
                "workspace.focus_previous",
                "Região anterior",
                ActionId::FocusPrevious,
            ),
        ] {
            let shortcut = self
                .session
                .shortcuts()
                .combo_for(action)
                .map(|combo| combo.display())
                .unwrap_or_default();
            rows.push(json!({"id":id,"label":label,"shortcut":shortcut,"enabled":true,"reason":"","rebindable":true,"shell":true}));
        }
        rows
    }

    pub(super) fn overlay_values(&self, primitives: Vec<OverlayPrimitive>) -> Vec<Value> {
        primitives.into_iter().map(|primitive|match primitive{
            OverlayPrimitive::Line{from,to}=>json!({"type":"line","x":from.x,"y":from.y,"x2":to.x,"y2":to.y}),
            OverlayPrimitive::Rect{x,y,width,height}=>json!({"type":"marquee","x":x,"y":y,"width":width,"height":height}),
            OverlayPrimitive::Handle{at}=>json!({"type":"handle","x":at.x,"y":at.y,"size":6}),
            OverlayPrimitive::Ghost{points}=>json!({"type":"ghost","points":points.into_iter().map(|point|self.screen_point(point)).map(|point|json!({"x":point.x,"y":point.y})).collect::<Vec<_>>()}),
        }).collect()
    }

    fn screen_point(&self, point: Point) -> Point {
        let view = self.session.view();
        petunia_ui::ViewTransform {
            scale: view.scale,
            rotation: view.rotation,
            offset_x: view.pan_x,
            offset_y: view.pan_y,
        }
        .doc_to_view(point)
    }

    fn current_overlays(&self) -> Vec<Value> {
        let mut rows = self.overlays.clone();
        if let Some(bounds) = self.selection_bounds() {
            let minimum = self.screen_point(bounds.min());
            let maximum = self.screen_point(bounds.max());
            rows.push(json!({"type":"bounds","x":minimum.x,"y":minimum.y,"width":maximum.x-minimum.x,"height":maximum.y-minimum.y}));
        }
        if self.session.contexts().in_vector() || self.session.active_tool() == ToolKind::NodeEdit {
            for id in self.session.selection().objects() {
                let Some(node) = self.session.document().scene.get_node(*id) else {
                    continue;
                };
                let Some(path) = node.item_path() else {
                    continue;
                };
                let Some(world) = self.session.document().scene.world_transform(*id) else {
                    continue;
                };
                for (contour_index, contour) in path.contours.iter().enumerate() {
                    for (node_index, point) in contour.nodes.iter().enumerate() {
                        let target = petunia_ui::NodeId {
                            object: *id,
                            contour: contour_index as u32,
                            node: node_index as u32,
                        };
                        let selected = self.session.selection().sub.nodes().contains(&target);
                        let anchor = self.screen_point(world.transform_point(point.point));
                        let node_type = match point.kind {
                            petunia_core::NodeKind::Cusp => "cusp",
                            petunia_core::NodeKind::Smooth => "smooth",
                            petunia_core::NodeKind::Symmetric => "symmetric",
                        };
                        rows.push(json!({"type":"node","x":anchor.x,"y":anchor.y,"size":6,"nodeType":node_type,"shape":node_type,"selected":selected}));
                        for handle in [point.handle_in, point.handle_out].into_iter().flatten() {
                            let end = self.screen_point(world.transform_point(handle));
                            rows.push(json!({"type":"handleLine","x":anchor.x,"y":anchor.y,"x2":end.x,"y2":end.y}));
                            rows.push(json!({"type":"handle","x":end.x,"y":end.y,"size":6,"selected":selected}));
                        }
                    }
                }
            }
        }
        rows
    }
}

fn history_label(description: HistoryDescription) -> &'static str {
    match description {
        HistoryDescription::MoveObjects => "Mover seleção",
        HistoryDescription::SetVisibility => "Alterar visibilidade",
        HistoryDescription::EditObjects => "Editar objetos",
        HistoryDescription::DeleteObjects => "Excluir seleção",
        HistoryDescription::SetFill => "Alterar preenchimento",
        HistoryDescription::ConvertToCurves => "Converter em curvas",
        HistoryDescription::InsertObjects => "Inserir objetos",
        HistoryDescription::SetTransform => "Transformar seleção",
    }
}
