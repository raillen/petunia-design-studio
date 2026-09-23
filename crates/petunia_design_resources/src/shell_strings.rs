//! Canonical shell and menu string catalog (08.2, 08.19, 09.16, 15.G).
//!
//! Every `ptnd.text.*` TextId the surface registry references resolves here,
//! in the canonical `en-US` source locale and its synchronized `pt-BR` mirror
//! (12.6). Feature code and UI must reference the TextId; the string itself
//! lives only in this table, which is what makes "no user-visible string
//! hardcoded in UI/feature logic" checkable rather than aspirational.
//!
//! Identifiers, action ids and schema values never appear here: they are
//! invariant across locales.

/// One localized pair for a stable `TextId`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShellString {
    /// Stable `ptnd.text.*` identifier.
    pub id: &'static str,
    /// Canonical `en-US` message.
    pub en: &'static str,
    /// Synchronized `pt-BR` message.
    pub pt: &'static str,
}

/// Declares one catalog row.
const fn entry(id: &'static str, en: &'static str, pt: &'static str) -> ShellString {
    ShellString { id, en, pt }
}

/// The canonical shell/menu catalog, sorted by id.
pub const SHELL_STRINGS: &[ShellString] = &[
    entry(
        "ptnd.text.blocked.action_unknown",
        "This control is not wired to an action",
        "Este controle não está ligado a uma ação",
    ),
    entry(
        "ptnd.text.blocked.data_merge_post_v1",
        "Data merge is planned after V1; the engine already exists",
        "A mesclagem de dados está planejada para depois da V1; o motor já existe",
    ),
    entry(
        "ptnd.text.blocked.distribute_three",
        "Distributing needs at least three objects",
        "Distribuir exige ao menos três objetos",
    ),
    entry(
        "ptnd.text.blocked.no_document",
        "No document is open",
        "Nenhum documento está aberto",
    ),
    entry(
        "ptnd.text.blocked.no_rule",
        "This action has no availability rule yet",
        "Esta ação ainda não tem regra de disponibilidade",
    ),
    entry(
        "ptnd.text.blocked.no_surface_to_fit",
        "There is no artboard to fit",
        "Não há prancheta para ajustar",
    ),
    entry(
        "ptnd.text.blocked.no_unsaved_changes",
        "The document has no unsaved changes",
        "O documento não tem alterações não salvas",
    ),
    entry(
        "ptnd.text.blocked.not_implemented",
        "This action is not implemented yet",
        "Esta ação ainda não está implementada",
    ),
    entry(
        "ptnd.text.blocked.nothing_selected",
        "Nothing is selected",
        "Nada está selecionado",
    ),
    entry(
        "ptnd.text.blocked.nothing_to_redo",
        "Nothing to redo",
        "Nada para refazer",
    ),
    entry(
        "ptnd.text.blocked.nothing_to_undo",
        "Nothing to undo",
        "Nada para desfazer",
    ),
    entry(
        "ptnd.text.blocked.offset_path",
        "Offset Path needs a distance the user types, and no numeric prompt exists yet",
        "Deslocar caminho precisa de uma distância digitada e ainda não existe campo numérico",
    ),
    entry(
        "ptnd.text.blocked.place_image",
        "Placing an asset needs an image object in the document model, which does not exist yet",
        "Inserir um recurso precisa de um objeto de imagem no modelo do documento, que ainda não existe",
    ),
    entry(
        "ptnd.text.blocked.point_transform",
        "Point Transform is folded into the Transform HUD",
        "Transformar ponto está incorporado ao HUD de transformação",
    ),
    entry(
        "ptnd.text.blocked.raster_post_v1",
        "Raster pixel layers are planned after V1",
        "Camadas de pixels raster estão planejadas para depois da V1",
    ),
    entry(
        "ptnd.text.blocked.select_object",
        "Select an object first",
        "Selecione um objeto primeiro",
    ),
    entry(
        "ptnd.text.blocked.select_two",
        "Select at least two objects",
        "Selecione ao menos dois objetos",
    ),
    entry(
        "ptnd.text.blocked.shape_builder",
        "Shape Builder is planned after V1",
        "O construtor de formas está planejado para depois da V1",
    ),
    entry(
        "ptnd.text.blocked.slice_path",
        "Slice Path needs a point the user picks on the path, and no tool supplies one yet",
        "Dividir caminho precisa de um ponto escolhido no caminho e nenhuma ferramenta fornece isso ainda",
    ),
    entry(
        "ptnd.text.canvas.viewport",
        "Canvas viewport",
        "Área de desenho",
    ),
    entry(
        "ptnd.text.dialog.about",
        "About Petunia Design Studio",
        "Sobre o Petunia Design Studio",
    ),
    entry("ptnd.text.dialog.export", "Export", "Exportar"),
    entry(
        "ptnd.text.dialog.new_document",
        "New Document",
        "Novo documento",
    ),
    entry(
        "ptnd.text.dialog.overwrite",
        "File already exists",
        "O arquivo já existe",
    ),
    entry("ptnd.text.dock.bottom", "Bottom dock", "Doca inferior"),
    entry("ptnd.text.dock.left", "Left dock", "Doca esquerda"),
    entry("ptnd.text.dock.right", "Right dock", "Doca direita"),
    entry("ptnd.text.edit.copy", "Copy", "Copiar"),
    entry("ptnd.text.edit.cut", "Cut", "Recortar"),
    entry("ptnd.text.edit.delete", "Delete", "Excluir"),
    entry("ptnd.text.edit.duplicate", "Duplicate", "Duplicar"),
    entry("ptnd.text.edit.paste", "Paste", "Colar"),
    entry(
        "ptnd.text.edit.preferences",
        "Preferences…",
        "Preferências…",
    ),
    entry("ptnd.text.edit.redo", "Redo", "Refazer"),
    entry("ptnd.text.edit.undo", "Undo", "Desfazer"),
    entry("ptnd.text.file.close", "Close Document", "Fechar documento"),
    entry("ptnd.text.file.export", "Export…", "Exportar…"),
    entry("ptnd.text.file.new", "New…", "Novo…"),
    entry("ptnd.text.file.open", "Open…", "Abrir…"),
    entry(
        "ptnd.text.file.open_recent",
        "Open Recent",
        "Abrir recentes",
    ),
    entry("ptnd.text.file.place", "Place Image…", "Inserir imagem…"),
    entry("ptnd.text.file.quit", "Quit", "Sair"),
    entry("ptnd.text.file.save", "Save", "Salvar"),
    entry("ptnd.text.file.save_as", "Save As…", "Salvar como…"),
    entry("ptnd.text.layer.arrange", "Arrange", "Organizar"),
    entry("ptnd.text.menu.bar", "Menu bar", "Barra de menus"),
    entry("ptnd.text.menu.edit", "Edit", "Editar"),
    entry("ptnd.text.menu.file", "File", "Arquivo"),
    entry("ptnd.text.menu.image", "Image", "Imagem"),
    entry("ptnd.text.menu.layer", "Layer", "Camada"),
    entry("ptnd.text.menu.object", "Object", "Objeto"),
    entry("ptnd.text.menu.select", "Select", "Selecionar"),
    entry("ptnd.text.menu.vector", "Vector", "Vetor"),
    entry("ptnd.text.menu.view", "View", "Exibir"),
    entry("ptnd.text.object.align", "Align", "Alinhar"),
    entry(
        "ptnd.text.object.align.bottom",
        "Align Bottom",
        "Alinhar à base",
    ),
    entry(
        "ptnd.text.object.align.center",
        "Align Centre",
        "Alinhar ao centro",
    ),
    entry(
        "ptnd.text.object.align.left",
        "Align Left",
        "Alinhar à esquerda",
    ),
    entry(
        "ptnd.text.object.align.middle",
        "Align Middle",
        "Alinhar ao meio",
    ),
    entry(
        "ptnd.text.object.align.right",
        "Align Right",
        "Alinhar à direita",
    ),
    entry("ptnd.text.object.align.top", "Align Top", "Alinhar ao topo"),
    entry("ptnd.text.object.arrange", "Arrange", "Organizar"),
    entry(
        "ptnd.text.object.arrange.back",
        "Send to Back",
        "Enviar para trás",
    ),
    entry(
        "ptnd.text.object.arrange.front",
        "Bring to Front",
        "Trazer para frente",
    ),
    entry(
        "ptnd.text.object.bake_corners",
        "Bake Corners",
        "Bake cantos",
    ),
    entry("ptnd.text.object.boolean", "Boolean", "Booleano"),
    entry(
        "ptnd.text.object.boolean.difference",
        "Subtract",
        "Subtração",
    ),
    entry("ptnd.text.object.boolean.exclusion", "Exclude", "Exclusão"),
    entry(
        "ptnd.text.object.boolean.intersection",
        "Intersect",
        "Interseção",
    ),
    entry("ptnd.text.object.boolean.union", "Union", "União"),
    entry(
        "ptnd.text.object.clip_mask",
        "Create Clipping Mask",
        "Criar máscara de recorte",
    ),
    entry(
        "ptnd.text.object.clip_mask_release",
        "Release Clipping Mask",
        "Liberar máscara de recorte",
    ),
    entry(
        "ptnd.text.object.convert_to_curves",
        "Convert to Curves",
        "Converter em curvas",
    ),
    entry("ptnd.text.object.distribute", "Distribute", "Distribuir"),
    entry(
        "ptnd.text.object.distribute.horizontal",
        "Distribute Horizontally",
        "Distribuir horizontalmente",
    ),
    entry(
        "ptnd.text.object.distribute.vertical",
        "Distribute Vertically",
        "Distribuir verticalmente",
    ),
    entry("ptnd.text.object.group", "Group", "Agrupar"),
    entry("ptnd.text.object.hide", "Hide", "Ocultar"),
    entry("ptnd.text.object.lock", "Lock", "Bloquear"),
    entry(
        "ptnd.text.object.offset_path",
        "Offset Path",
        "Deslocar caminho",
    ),
    entry(
        "ptnd.text.object.slice_path",
        "Slice Path",
        "Dividir caminho",
    ),
    entry("ptnd.text.object.ungroup", "Ungroup", "Desagrupar"),
    entry("ptnd.text.panel.align", "Align", "Alinhar"),
    entry("ptnd.text.panel.appearance", "Appearance", "Aparência"),
    entry("ptnd.text.panel.assets", "Assets", "Recursos"),
    entry("ptnd.text.panel.color", "Colour", "Cor"),
    entry(
        "ptnd.text.panel.data_merge",
        "Data Merge",
        "Mesclagem de dados",
    ),
    entry("ptnd.text.panel.fill", "Fill", "Preenchimento"),
    entry("ptnd.text.panel.history", "History", "Histórico"),
    entry(
        "ptnd.text.panel.jobs",
        "Background Tasks",
        "Tarefas em segundo plano",
    ),
    entry("ptnd.text.panel.layers", "Layers", "Camadas"),
    entry("ptnd.text.panel.navigator", "Navigator", "Navegador"),
    entry("ptnd.text.panel.properties", "Properties", "Propriedades"),
    entry("ptnd.text.panel.stroke", "Stroke", "Traçado"),
    entry("ptnd.text.panel.swatches", "Swatches", "Amostras"),
    entry("ptnd.text.panel.transform", "Transform", "Transformar"),
    entry(
        "ptnd.text.persona.photo",
        "Photo Persona",
        "Persona Foto",
    ),
    entry(
        "ptnd.text.persona.photo.hint",
        "Photo Persona: pixel painting, cropping, retouching and raster masks",
        "Persona Foto: pintura de pixels, recorte, retoque e máscaras raster",
    ),
    entry(
        "ptnd.text.persona.vector",
        "Vector Persona",
        "Persona Vetorial",
    ),
    entry(
        "ptnd.text.persona.vector.hint",
        "Vector Persona: drawing, nodes, fills and curves",
        "Persona Vetorial: desenho, nós, preenchimentos e curvas",
    ),
    entry("ptnd.text.select.all", "Select All", "Selecionar tudo"),
    entry(
        "ptnd.text.select.invert",
        "Invert Selection",
        "Inverter seleção",
    ),
    entry("ptnd.text.select.none", "Deselect", "Desmarcar"),
    entry(
        "ptnd.text.shell.brand",
        "Petunia Design Studio",
        "Petunia Design Studio",
    ),
    entry("ptnd.text.shell.brand_mark", "Brand mark", "Marca"),
    entry(
        "ptnd.text.shell.context_toolbar",
        "Context toolbar",
        "Barra de contexto",
    ),
    entry(
        "ptnd.text.shell.customize",
        "Customize toolbar",
        "Personalizar barra",
    ),
    entry("ptnd.text.shell.divider", "Divider", "Divisor"),
    entry("ptnd.text.shell.export", "Export", "Exportar"),
    entry(
        "ptnd.text.shell.move_down",
        "Move down",
        "Mover para baixo",
    ),
    entry("ptnd.text.shell.move_up", "Move up", "Mover para cima"),
    entry("ptnd.text.shell.overflow", "More actions", "Mais ações"),
    entry(
        "ptnd.text.shell.palette",
        "Command Palette",
        "Paleta de comandos",
    ),
    entry("ptnd.text.shell.persona", "Persona", "Persona"),
    entry("ptnd.text.shell.redo", "Redo", "Refazer"),
    entry("ptnd.text.shell.remove", "Remove", "Remover"),
    entry(
        "ptnd.text.shell.reset_toolbar",
        "Reset toolbar",
        "Restaurar barra",
    ),
    entry("ptnd.text.shell.spacer", "Flexible space", "Espaço flexível"),
    entry(
        "ptnd.text.shell.status_bar",
        "Status bar",
        "Barra de status",
    ),
    entry(
        "ptnd.text.shell.tool_rail",
        "Tool rail",
        "Coluna de ferramentas",
    ),
    entry("ptnd.text.shell.undo", "Undo", "Desfazer"),
    entry(
        "ptnd.text.shell.zoom_readout",
        "Zoom level",
        "Nível de zoom",
    ),
    entry("ptnd.text.tabs.close", "Close tab", "Fechar aba"),
    entry(
        "ptnd.text.tabs.dirty",
        "Unsaved changes",
        "Alterações não salvas",
    ),
    entry(
        "ptnd.text.tabs.strip",
        "Document tabs",
        "Abas de documentos",
    ),
    entry(
        "ptnd.text.tool.artistic_text",
        "Artistic Text",
        "Texto artístico",
    ),
    entry("ptnd.text.tool.brush", "Brush", "Pincel"),
    entry("ptnd.text.tool.contour", "Contour", "Contorno"),
    entry("ptnd.text.tool.corner", "Corner", "Canto"),
    entry("ptnd.text.tool.crop", "Crop", "Recortar"),
    entry("ptnd.text.tool.ellipse", "Ellipse", "Elipse"),
    entry("ptnd.text.tool.eraser", "Eraser", "Borracha"),
    entry("ptnd.text.tool.eyedropper", "Eyedropper", "Conta-gotas"),
    entry("ptnd.text.tool.frame_text", "Frame Text", "Texto em quadro"),
    entry("ptnd.text.tool.gradient", "Gradient", "Degradê"),
    entry("ptnd.text.tool.hand", "Hand", "Mão"),
    entry("ptnd.text.tool.knife", "Knife", "Faca"),
    entry("ptnd.text.tool.line", "Line", "Linha"),
    entry("ptnd.text.tool.move", "Move", "Mover"),
    entry("ptnd.text.tool.node", "Node", "Nó"),
    entry("ptnd.text.tool.pen", "Pen", "Caneta"),
    entry("ptnd.text.tool.pencil", "Pencil", "Lápis"),
    entry(
        "ptnd.text.tool.place_image",
        "Place Image",
        "Inserir imagem",
    ),
    entry(
        "ptnd.text.tool.point_transform",
        "Point Transform",
        "Transformar ponto",
    ),
    entry("ptnd.text.tool.polygon", "Polygon", "Polígono"),
    entry("ptnd.text.tool.rectangle", "Rectangle", "Retângulo"),
    entry("ptnd.text.tool.scissors", "Scissors", "Tesoura"),
    entry(
        "ptnd.text.tool.shape_builder",
        "Shape Builder",
        "Construtor de formas",
    ),
    entry("ptnd.text.tool.star", "Star", "Estrela"),
    entry("ptnd.text.tool.surface", "Artboard", "Prancheta"),
    entry(
        "ptnd.text.tool.transparency",
        "Transparency",
        "Transparência",
    ),
    entry(
        "ptnd.text.tool.vector_brush",
        "Vector Brush",
        "Pincel vetorial",
    ),
    entry("ptnd.text.tool.zoom", "Zoom", "Zoom"),
    entry(
        "ptnd.text.view.command_palette",
        "Command Palette",
        "Paleta de comandos",
    ),
    entry(
        "ptnd.text.view.fit_surface",
        "Fit Artboard",
        "Ajustar à prancheta",
    ),
    entry(
        "ptnd.text.view.focus_canvas",
        "Focus Canvas",
        "Focar a área de desenho",
    ),
    entry("ptnd.text.view.rulers", "Rulers", "Réguas"),
    entry("ptnd.text.view.snapping", "Snapping", "Encaixe"),
    entry("ptnd.text.view.zoom_100", "100%", "100%"),
    entry("ptnd.text.view.zoom_200", "200%", "200%"),
    entry("ptnd.text.view.zoom_25", "25%", "25%"),
    entry("ptnd.text.view.zoom_400", "400%", "400%"),
    entry("ptnd.text.view.zoom_50", "50%", "50%"),
    entry("ptnd.text.view.zoom_800", "800%", "800%"),
    entry("ptnd.text.view.zoom_in", "Zoom In", "Aproximar"),
    entry("ptnd.text.view.zoom_levels", "Zoom Levels", "Níveis de zoom"),
    entry("ptnd.text.view.zoom_out", "Zoom Out", "Afastar"),
    entry("ptnd.text.window.home", "Home", "Início"),
    entry("ptnd.text.window.plugins", "Plugins", "Plugins"),
    entry(
        "ptnd.text.window.preferences",
        "Preferences",
        "Preferências",
    ),
];

/// Looks up one catalog row by TextId.
#[must_use]
pub fn shell_string(id: &str) -> Option<&'static ShellString> {
    SHELL_STRINGS.iter().find(|entry| entry.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn identifiers_are_unique_and_namespaced() {
        let mut seen = HashSet::new();
        for entry in SHELL_STRINGS {
            assert!(
                entry.id.starts_with("ptnd.text."),
                "`{}` is not a ptnd.text.* TextId",
                entry.id
            );
            assert!(seen.insert(entry.id), "duplicate TextId `{}`", entry.id);
        }
    }

    #[test]
    fn both_locales_are_populated() {
        for entry in SHELL_STRINGS {
            assert!(
                !entry.en.trim().is_empty(),
                "`{}` has no en-US string",
                entry.id
            );
            assert!(
                !entry.pt.trim().is_empty(),
                "`{}` has no pt-BR string",
                entry.id
            );
        }
    }

    #[test]
    fn the_table_is_sorted_so_reviews_stay_reviewable() {
        let mut sorted = SHELL_STRINGS.to_vec();
        sorted.sort_by_key(|entry| entry.id);
        assert_eq!(
            sorted,
            SHELL_STRINGS.to_vec(),
            "catalog must stay sorted by id"
        );
    }

    #[test]
    fn lookup_resolves_and_rejects() {
        assert_eq!(
            shell_string("ptnd.text.file.save").map(|entry| entry.pt),
            Some("Salvar")
        );
        assert!(shell_string("ptnd.text.does.not.exist").is_none());
    }
}
