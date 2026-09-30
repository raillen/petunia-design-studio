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
        "ptnd.text.appearance.accent_color",
        "Accent color",
        "Cor de destaque",
    ),
    entry(
        "ptnd.text.appearance.icon_style",
        "Icon style",
        "Estilo dos ícones",
    ),
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
        "Slice Path needs a point picked on the path: click it with the Scissors tool",
        "Dividir caminho precisa de um ponto escolhido no caminho: clique nele com a ferramenta Tesoura",
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
    entry("ptnd.text.shell.add_tool", "Add tool", "Adicionar ferramenta"),
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
    entry("ptnd.text.shell.hidden", "Hidden", "Oculto"),
    entry(
        "ptnd.text.shell.hide_toolbar_item",
        "Hide toolbar item",
        "Ocultar item da barra",
    ),
    entry("ptnd.text.shell.merge_group", "Merge", "Mesclar"),
    entry("ptnd.text.shell.more_tools", "More tools", "Mais ferramentas"),
    entry(
        "ptnd.text.shell.move_down",
        "Move down",
        "Mover para baixo",
    ),
    entry("ptnd.text.shell.move_up", "Move up", "Mover para cima"),
    entry("ptnd.text.shell.next_group", "Next group", "Próximo grupo"),
    entry("ptnd.text.shell.open_menu", "Open menu", "Abrir menu"),
    entry("ptnd.text.shell.overflow", "More actions", "Mais ações"),
    entry(
        "ptnd.text.shell.palette",
        "Command Palette",
        "Paleta de comandos",
    ),
    entry("ptnd.text.shell.persona", "Persona", "Persona"),
    entry(
        "ptnd.text.shell.previous_group",
        "Previous group",
        "Grupo anterior",
    ),
    entry("ptnd.text.shell.redo", "Redo", "Refazer"),
    entry("ptnd.text.shell.remove", "Remove", "Remover"),
    entry(
        "ptnd.text.shell.reset_toolbar",
        "Reset toolbar",
        "Restaurar barra",
    ),
    entry(
        "ptnd.text.shell.show_toolbar_item",
        "Show toolbar item",
        "Mostrar item da barra",
    ),
    entry("ptnd.text.shell.snap_off", "Off", "Desligado"),
    entry("ptnd.text.shell.snap_on", "On", "Ligado"),
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
    entry(
        "ptnd.text.shell.tool_rail_one_column",
        "1 column",
        "1 coluna",
    ),
    entry(
        "ptnd.text.shell.tool_rail_responsive",
        "Responsive rail",
        "Coluna responsiva",
    ),
    entry(
        "ptnd.text.shell.tool_rail_responsive_one",
        "Responsive rail: one column",
        "Coluna responsiva: uma coluna",
    ),
    entry(
        "ptnd.text.shell.tool_rail_responsive_two",
        "Responsive rail: two columns",
        "Coluna responsiva: duas colunas",
    ),
    entry(
        "ptnd.text.shell.tool_rail_two_columns",
        "2 columns",
        "2 colunas",
    ),
    entry("ptnd.text.shell.undo", "Undo", "Desfazer"),
    entry("ptnd.text.shell.visible", "Visible", "Visível"),
    entry(
        "ptnd.text.shell.zoom_readout",
        "Zoom level",
        "Nível de zoom",
    ),
    entry(
        "ptnd.text.summary.add_divider",
        "Add a visual separator",
        "Adicionar um separador visual",
    ),
    entry(
        "ptnd.text.summary.choose_accent",
        "Choose the workspace accent color",
        "Escolher a cor de destaque do espaço de trabalho",
    ),
    entry(
        "ptnd.text.summary.command",
        "Run this command",
        "Executar este comando",
    ),
    entry(
        "ptnd.text.summary.customize_layout",
        "Organize tools and toolbar layout",
        "Organize ferramentas e layout da barra",
    ),
    entry(
        "ptnd.text.summary.delete",
        "Delete the selected objects",
        "Excluir os objetos selecionados",
    ),
    entry(
        "ptnd.text.summary.edit_action",
        "Edit the current document",
        "Editar o documento atual",
    ),
    entry(
        "ptnd.text.summary.export",
        "Export the current document",
        "Exportar o documento atual",
    ),
    entry(
        "ptnd.text.summary.file_action",
        "Work with the current file",
        "Trabalhar com o arquivo atual",
    ),
    entry(
        "ptnd.text.summary.fit",
        "Fit the active artboard in view",
        "Ajustar a prancheta ativa à tela",
    ),
    entry(
        "ptnd.text.summary.object_action",
        "Change the selected objects",
        "Alterar os objetos selecionados",
    ),
    entry(
        "ptnd.text.summary.redo",
        "Redo the last change",
        "Refazer a última alteração",
    ),
    entry(
        "ptnd.text.summary.reorder_toolbar",
        "Reorder this toolbar item",
        "Reordenar este item da barra",
    ),
    entry(
        "ptnd.text.summary.search_commands",
        "Search available commands",
        "Pesquisar comandos disponíveis",
    ),
    entry(
        "ptnd.text.summary.shell_action",
        "Change the shell view",
        "Alterar a visualização do shell",
    ),
    entry(
        "ptnd.text.summary.toggle_icon_style",
        "Switch between outline and filled icons",
        "Alternar entre ícones de contorno e preenchidos",
    ),
    entry(
        "ptnd.text.summary.toggle_toolbar_item",
        "Click the row to toggle visibility",
        "Clique na linha para alternar a visibilidade",
    ),
    entry(
        "ptnd.text.summary.tool_action",
        "Activate a tool",
        "Ativar uma ferramenta",
    ),
    entry(
        "ptnd.text.summary.toolbar_action",
        "Apply this toolbar command",
        "Aplicar este comando da barra",
    ),
    entry(
        "ptnd.text.summary.undo",
        "Undo the last change",
        "Desfazer a última alteração",
    ),
    entry(
        "ptnd.text.summary.view_action",
        "Change the canvas view",
        "Alterar a visualização da tela",
    ),
    entry(
        "ptnd.text.summary.zoom_in",
        "Zoom in on the canvas",
        "Aproximar a tela",
    ),
    entry(
        "ptnd.text.summary.zoom_out",
        "Zoom out on the canvas",
        "Afastar a tela",
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
    entry(
        "ptnd.text.tool.artistic_text.summary",
        "Create expressive typography",
        "Crie tipografia expressiva",
    ),
    entry("ptnd.text.tool.brush", "Brush", "Pincel"),
    entry(
        "ptnd.text.tool.brush.summary",
        "Paint pixels on a raster layer",
        "Pinte pixels em uma camada raster",
    ),
    entry("ptnd.text.tool.contour", "Contour", "Contorno"),
    entry(
        "ptnd.text.tool.contour.summary",
        "Expand or inset a live outline",
        "Expanda ou contraia um contorno ao vivo",
    ),
    entry("ptnd.text.tool.corner", "Corner", "Canto"),
    entry(
        "ptnd.text.tool.corner.summary",
        "Edit a corner radius directly",
        "Edite o raio do canto diretamente",
    ),
    entry("ptnd.text.tool.crop", "Crop", "Recortar"),
    entry(
        "ptnd.text.tool.crop.summary",
        "Crop a surface or raster layer",
        "Recorte uma superfície ou camada raster",
    ),
    entry("ptnd.text.tool.ellipse", "Ellipse", "Elipse"),
    entry(
        "ptnd.text.tool.ellipse.summary",
        "Draw an ellipse or circle",
        "Desenhe uma elipse ou círculo",
    ),
    entry("ptnd.text.tool.eraser", "Eraser", "Borracha"),
    entry(
        "ptnd.text.tool.eraser.summary",
        "Erase raster pixels or masks",
        "Apague pixels raster ou máscaras",
    ),
    entry("ptnd.text.tool.eyedropper", "Eyedropper", "Conta-gotas"),
    entry(
        "ptnd.text.tool.eyedropper.summary",
        "Sample a color from the canvas",
        "Amostre uma cor da área de desenho",
    ),
    entry("ptnd.text.tool.flood_select", "Flood Select", "Seleção por cor"),
    entry(
        "ptnd.text.tool.flood_select.summary",
        "Select contiguous similar pixels",
        "Selecione pixels contíguos semelhantes",
    ),
    entry("ptnd.text.tool.frame_text", "Frame Text", "Texto em quadro"),
    entry(
        "ptnd.text.tool.frame_text.summary",
        "Create text inside a frame",
        "Crie texto dentro de um quadro",
    ),
    entry("ptnd.text.tool.gradient", "Gradient", "Degradê"),
    entry(
        "ptnd.text.tool.gradient.summary",
        "Paint a vector or raster gradient",
        "Pinte um degradê vetorial ou raster",
    ),
    entry("ptnd.text.tool.hand", "Hand", "Mão"),
    entry(
        "ptnd.text.tool.hand.summary",
        "Pan the canvas without changing tools",
        "Mova a tela sem trocar a ferramenta",
    ),
    entry("ptnd.text.tool.knife", "Knife", "Faca"),
    entry(
        "ptnd.text.tool.knife.summary",
        "Cut a path with a freehand gesture",
        "Corte um caminho com um gesto livre",
    ),
    entry("ptnd.text.tool.lasso", "Lasso", "Laço"),
    entry(
        "ptnd.text.tool.lasso.summary",
        "Select pixels with a freehand path",
        "Selecione pixels com um caminho livre",
    ),
    entry("ptnd.text.tool.line", "Line", "Linha"),
    entry("ptnd.text.tool.marquee_ellipse", "Elliptical Marquee", "Marquee elíptico"),
    entry(
        "ptnd.text.tool.marquee_ellipse.summary",
        "Select pixels inside an ellipse",
        "Selecione pixels dentro de uma elipse",
    ),
    entry("ptnd.text.tool.marquee_rect", "Rectangular Marquee", "Marquee retangular"),
    entry(
        "ptnd.text.tool.marquee_rect.summary",
        "Select pixels inside a rectangle",
        "Selecione pixels dentro de um retângulo",
    ),
    entry("ptnd.text.tool.measure", "Measure", "Medir"),
    entry(
        "ptnd.text.tool.measure.summary",
        "Measure distance, angle, or area",
        "Meça distância, ângulo ou área",
    ),
    entry("ptnd.text.tool.move", "Move", "Mover"),
    entry(
        "ptnd.text.tool.move.summary",
        "Select and move objects",
        "Selecione e mova objetos",
    ),
    entry("ptnd.text.tool.node", "Node", "Nó"),
    entry(
        "ptnd.text.tool.node.summary",
        "Edit path anchors and handles",
        "Edite pontos e alças de caminhos",
    ),
    entry("ptnd.text.tool.pen", "Pen", "Caneta"),
    entry(
        "ptnd.text.tool.pen.summary",
        "Build precise Bézier paths",
        "Construa caminhos Bézier precisos",
    ),
    entry("ptnd.text.tool.pencil", "Pencil", "Lápis"),
    entry(
        "ptnd.text.tool.pencil.summary",
        "Sketch and refine freehand paths",
        "Desenhe e refine caminhos livres",
    ),
    entry(
        "ptnd.text.tool.perspective",
        "Perspective",
        "Perspectiva",
    ),
    entry(
        "ptnd.text.tool.perspective.summary",
        "Warp an object with four corners",
        "Deforme um objeto com quatro cantos",
    ),
    entry(
        "ptnd.text.tool.place_image",
        "Place Image",
        "Inserir imagem",
    ),
    entry(
        "ptnd.text.tool.place_image.summary",
        "Place an image file onto the active artboard",
        "Insira um arquivo de imagem na prancheta ativa",
    ),
    entry(
        "ptnd.text.tool.point_transform",
        "Point Transform",
        "Transformar ponto",
    ),
    entry(
        "ptnd.text.tool.point_transform.summary",
        "Move a custom transform pivot",
        "Mova um pivô de transformação personalizado",
    ),
    entry("ptnd.text.tool.polygon", "Polygon", "Polígono"),
    entry(
        "ptnd.text.tool.polygon.summary",
        "Draw a regular polygon",
        "Desenhe um polígono regular",
    ),
    entry("ptnd.text.tool.rectangle", "Rectangle", "Retângulo"),
    entry(
        "ptnd.text.tool.rectangle.summary",
        "Draw a rectangle with optional corners",
        "Desenhe um retângulo com cantos opcionais",
    ),
    entry("ptnd.text.tool.scissors", "Scissors", "Tesoura"),
    entry(
        "ptnd.text.tool.scissors.summary",
        "Split a path at a hit point",
        "Divida um caminho em um ponto",
    ),
    entry("ptnd.text.tool.select", "Select", "Selecionar"),
    entry(
        "ptnd.text.tool.select.summary",
        "Select and transform objects",
        "Selecione e transforme objetos",
    ),
    entry("ptnd.text.tool.selection_brush", "Selection Brush", "Pincel de seleção"),
    entry(
        "ptnd.text.tool.selection_brush.summary",
        "Paint a raster selection mask",
        "Pinte uma máscara de seleção raster",
    ),
    entry(
        "ptnd.text.tool.shape_builder",
        "Shape Builder",
        "Construtor de formas",
    ),
    entry(
        "ptnd.text.tool.shape_builder.summary",
        "Combine and subtract vector regions",
        "Combine e subtraia regiões vetoriais",
    ),
    entry("ptnd.text.tool.star", "Star", "Estrela"),
    entry(
        "ptnd.text.tool.star.summary",
        "Draw a star with editable points",
        "Desenhe uma estrela com pontos editáveis",
    ),
    entry("ptnd.text.tool.style_picker", "Style Picker", "Seletor de estilo"),
    entry(
        "ptnd.text.tool.style_picker.summary",
        "Sample a complete appearance style",
        "Amostre um estilo de aparência completo",
    ),
    entry("ptnd.text.tool.surface", "Artboard", "Prancheta"),
    entry(
        "ptnd.text.tool.surface.summary",
        "Create and resize artboards",
        "Crie e redimensione pranchetas",
    ),
    entry(
        "ptnd.text.tool.transparency",
        "Transparency",
        "Transparência",
    ),
    entry(
        "ptnd.text.tool.transparency.summary",
        "Edit a live transparency gradient",
        "Edite um degradê de transparência ao vivo",
    ),
    entry(
        "ptnd.text.tool.vector_brush",
        "Vector Brush",
        "Pincel vetorial",
    ),
    entry(
        "ptnd.text.tool.vector_brush.summary",
        "Paint a vector brush stroke",
        "Pinte um traço de pincel vetorial",
    ),
    entry(
        "ptnd.text.tool.vector_flood_fill",
        "Vector Smart Fill",
        "Preenchimento vetorial",
    ),
    entry(
        "ptnd.text.tool.vector_flood_fill.summary",
        "Fill a bounded vector region",
        "Preencha uma região vetorial limitada",
    ),
    entry("ptnd.text.tool.zoom", "Zoom", "Zoom"),
    entry(
        "ptnd.text.tool.zoom.summary",
        "Zoom the canvas around the pointer",
        "Amplie a tela ao redor do ponteiro",
    ),
    entry(
        "ptnd.text.tool_group.artboard",
        "Artboard",
        "Prancheta",
    ),
    entry(
        "ptnd.text.tool_group.crop",
        "Crop",
        "Recorte",
    ),
    entry(
        "ptnd.text.tool_group.cut",
        "Cut",
        "Corte",
    ),
    entry(
        "ptnd.text.tool_group.gradient_transparency",
        "Gradient / Transparency",
        "Degradê / Transparência",
    ),
    entry(
        "ptnd.text.tool_group.hand",
        "Hand",
        "Mão",
    ),
    entry(
        "ptnd.text.tool_group.measure",
        "Measure",
        "Medida",
    ),
    entry(
        "ptnd.text.tool_group.modify",
        "Modify",
        "Modificar",
    ),
    entry(
        "ptnd.text.tool_group.pen",
        "Pen",
        "Caneta",
    ),
    entry(
        "ptnd.text.tool_group.pencil",
        "Pencil",
        "Lápis",
    ),
    entry(
        "ptnd.text.tool_group.photo_content",
        "Photo content",
        "Conteúdo de foto",
    ),
    entry(
        "ptnd.text.tool_group.photo_paint",
        "Photo paint",
        "Pintura de foto",
    ),
    entry(
        "ptnd.text.tool_group.photo_selection",
        "Photo selection",
        "Seleção de foto",
    ),
    entry(
        "ptnd.text.tool_group.pick",
        "Pick",
        "Selecionar estilo",
    ),
    entry(
        "ptnd.text.tool_group.selection",
        "Selection",
        "Seleção",
    ),
    entry(
        "ptnd.text.tool_group.shapes",
        "Shapes",
        "Formas",
    ),
    entry(
        "ptnd.text.tool_group.shared",
        "Shared tools",
        "Ferramentas compartilhadas",
    ),
    entry(
        "ptnd.text.tool_group.text",
        "Text",
        "Texto",
    ),
    entry(
        "ptnd.text.tool_group.zoom",
        "Zoom",
        "Zoom",
    ),
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
