# Ledger de Implementação da Interface — Petunia Design Studio

Documento de consulta para implementar a UI **item a item**, em vez de tudo de
uma vez. Cada linha é uma unidade de trabalho endereçável: tem ID estável,
posição na shell, o que já funciona e o que falta.

**Fontes da verdade (não improvise a partir deste arquivo):**

| Assunto | Fonte |
|---|---|
| Identidade, status e binding de cada superfície | `crates/petunia_design_application/src/surfaces.rs` (registry `15.G`) |
| O que a UI desenha | `apps/petunia-design/ui/app.slint` |
| Geometria e política de literais | `apps/petunia-design/ui/tokens.slint` (08.35, 08.21) |
| Wiring (callbacks → Action lane) | `apps/petunia-design/src/main.rs` |
| Strings (en-US + pt-BR) | `crates/petunia_design_resources/src/shell_strings.rs` |
| Menu derivado | `crates/petunia_design_application/src/menus.rs` |

> Este ledger **não é normativo**. Ele descreve o estado observado do código.
> Onde ele discordar de `15.A`–`15.H`, os documentos normativos ganham — e a
> discordância deve ser registrada (ver §15).

---

## 0. Como usar (fluxo por item)

1. Escolha **um** ID da tabela da região que você vai mexer.
2. Leia a coluna **"Falta"** dele — é o critério de aceite.
3. Confirme no registry se o status atual está correto (`Wired` / `Disabled(reason)` / `Absent`).
4. Implemente **um item por PR** atravessando a lane completa:
   UI → callback → `ActionRequest` → `Command` → `DocumentMutator` → `ChangeSet`.
5. Feche com a verificação de §17. Se o item mudou contrato, atualize o registry **no mesmo PR**.

**Regra de honestidade (15.F §2):** nenhum controle visível pode existir sem
comportamento. Ou funciona, ou aparece **desabilitado com a razão**, ou não
existe. Botão com `callback` vazio é bug, não dívida.

---

## 1. Legenda

**Status** (do registry, coluna oficial):

| Símbolo | Significado |
|---|---|
| ✅ `Wired` | Ligado a ação/tool real e exercitado |
| ⛔ `Disabled` | Renderizado mas inerte, **com motivo**. O motivo é obrigatório |
| ⬜ `Absent` | Não implementado. Não pode ser apresentado como funcional |

**Marcas deste ledger** (achados que o registry não captura):

| Marca | Significado |
|---|---|
| 🎭 **Fake UI** | Controle visível sem comportamento ou mentindo sobre o estado |
| ⚰️ **Código morto** | Callback/handler que existe mas nada invoca |
| 🌐 **i18n** | String literal user-facing que ainda não vem do catálogo |
| 📐 **Literal** | Geometria literal fora da política de 08.35 |

**Números de hoje:** 113 superfícies (79 ✅ / 8 ⛔ / 26 ⬜), 65 ações despacháveis,
6 famílias de menu com 41 itens, 19 botões de ferramenta, 41 atalhos declarados
(dos quais **9 ativos**).

---

## 2. Mapa da shell — posições e alturas canônicas

Ordem vertical do `MainWindow` (`app.slint`, `VerticalLayout` raiz):

| # | Região | Altura / largura | Seção do `app.slint` | Tokens |
|---|---|---|---|---|
| 1 | Menu bar + persona | `persona-row-height` **40px** | §1 (606) | `surface-chrome` |
| 2 | Tab strip de documento | `tab-strip-height + space-1` **34px** | §2 (690) | `surface-chrome-strong` |
| 3 | Context toolbar | `context-toolbar-height` **34px** | §3 (832) | `surface-panel` |
| 4 | Tool rail (esquerda) | `tool-rail-width` **44px** | §4.1 (1011) | `surface-chrome` |
| 4b | Régua vertical | **22px** | §4.1b (1161) | `surface-chrome-strong` |
| 4c | Régua horizontal | **18px** | §4.1c (1192) | `surface-chrome-strong` |
| 4d | Canvas viewport | flexível (dominante) | §4.2 (1222) | `surface-workspace` |
| 4e | Dock direito | `right-dock-default-width` **304px** | §4.3 (1417) | `surface-chrome` |
| 5 | Status bar | `status-bar-height` **26px** | §5 (1998) | `surface-chrome-strong` |
| — | Overlays | — | §5 export (2044), §6 palette (2208), §7 menu (2289) | — |

**Ausentes da shell:** dock esquerdo, dock inferior, splitter (1px visível / 6px hit).

---

## 3. Tabela A — Chrome da shell (18 superfícies)

| ID (`15.G`) | Posição | Função atual | Status | Falta |
|---|---|---|---|---|
| `ptnd.surface.shell.brand` | §1, esquerda | Slot de ícone quadrado (sem texto) | ✅ | Ícone simbólico ainda não existe; o slot apenas reserva o espaço |
| `ptnd.surface.menu_bar` | §1, centro | 6 famílias geradas do registry | ✅ | — |
| `ptnd.surface.shell.persona_persona` | §1, direita | Segmentos "Persona Vetorial" / "Persona Foto", **só texto**, por id do registry | ✅ | 🎭 troca **o menu Vetor/Imagem e o hint da status bar**; rail e painéis não mudam (§5) |
| `ptnd.surface.shell.undo` | §1, cluster central | `edit.undo`, `Ctrl+Z` | ✅ | — |
| `ptnd.surface.shell.redo` | §1, cluster central | `edit.redo`, `Ctrl+Y` | ✅ | — |
| `ptnd.surface.shell.zoom_out` / `zoom_readout` / `zoom_in` / `fit` / `divider` | §1, cluster central | Afastar, **caixa de níveis de zoom**, aproximar, ajustar, divisor | ✅ | O readout é um **controle**: abre o popup com os mesmos níveis do submenu `View ▸ Níveis de zoom` |
| `ptnd.surface.shell.command_palette` | overlay §6 | `Ctrl+K`, oferta = itens de menu habilitados | ✅ | 🌐 placeholder literal |
| `ptnd.surface.shell.export` | §3, direita | Abre o diálogo de export | ✅ | — |
| `ptnd.surface.shell.overflow` | — | — | ⬜ | Não existe. Menu bar não tem "…" para janelas estreitas |
| `ptnd.surface.tabs.document_strip` | §2 | 1 aba fixa | ✅ | 🎭 `clicked => {}` vazio; multi-documento real ausente |
| `ptnd.surface.tabs.document_close` | §2 | X desenhado no `StudioTabItem` | ⬜ | 🎭 **`close_clicked => {}` vazio** — o X aparece e não faz nada |
| `ptnd.surface.tabs.document_dirty` | §2 | Indicador "•" | ✅ | 🎭 `is_dirty: true` **hardcoded** — o "•" está sempre aceso |
| `ptnd.surface.context_toolbar` | §3 | Ver Tabela H | ✅ | — |
| `ptnd.surface.tool_rail` | §4.1 | 19 botões | ✅ | Ver Tabela C |
| `ptnd.surface.status_bar` | §5 | Ver Tabela I | ✅ | 🌐 textos literais/hardcoded |
| `ptnd.surface.dock.left` | — | — | ⬜ | **Não existe** (escopo `MilestoneRequired`) |
| `ptnd.surface.dock.right` | §4.3 | 3 abas | ✅ | Ver Tabela E |
| `ptnd.surface.dock.bottom` | — | — | ⬜ | **Não existe** (escopo `MilestoneRequired`) |
| `ptnd.surface.canvas.viewport` | §4.2 | Artboard, bleed, margens, guia, objetos, 8 handles, pin de rotação, HUD de dimensão, preview de arraste, cursor por ferramenta | ✅ | Sem zoom/pan real por scroll; sem réguas de coordenada reais além do offset |

---

## 4. Tabela B — Barra de menu (derivada do registry)

**7 famílias por persona**, geradas por `menus.rs::MENU_BAR` e escopadas por
`personas` — **Vetor** e **Imagem** são mutuamente exclusivas. A UI **não possui
label nem action id**: ambos chegam de `query_menu_bar()`. Contagem medida no
modelo resolvido (`menu_bar_for`, com documento aberto):

| Família | ID | Linhas | Itens | Persona | Submenus e itens |
|---|---|---|---|---|---|
| File | `ptnd.menu.file` | 6 | 6 | todas | new, open, save, save_as, export, place (⛔ com motivo) |
| Edit | `ptnd.menu.edit` | 4 | 4 | todas | undo, redo, duplicate, delete |
| Select | `ptnd.menu.select` | 2 | 2 | todas | select_all, deselect |
| Object | `ptnd.menu.object` | 6 | 12 | todas | group, ungroup, ▸Alinhamento (6), ▸Distribuir (2), lock, hide |
| Layer | `ptnd.menu.layer` | 2 | 4 | todas | ▸Organizar (2), ▸Máscara de recorte (2) |
| **Vetor** | `ptnd.menu.vector` | 6 | 18 | vetorial | convert_to_curves, bake_corners, ▸Booleano (4), ▸Traçado (2), ▸Formas (4), ▸Caneta (6) |
| **Imagem** | `ptnd.menu.image` | 1 | 5 | foto | ▸Transformar (5, todas ferramentas) |
| View | `ptnd.menu.view` | 7 | 12 | todas | zoom_in, zoom_out, ▸Níveis de zoom (6), fit_surface, rulers, snapping, palette |

Totais: **24 linhas / 58 itens** na Persona Vetorial; **22 / 45** na Persona Foto.

Um item de submenu é um item como qualquer outro: o mesmo token
(`surface#payload`) entra pela lane de Action, então a paleta de comandos e o
popup da caixa de zoom enxergam exatamente os mesmos itens que o menu. A
contagem de linhas conta a linha do submenu uma vez (a contagem de itens não).

**Falta na barra de menu**

| Item | Situação |
|---|---|
| `window` / `help` | Não existem (nada de `dialog.about`, `window.preferences`, `window.home`) |
| Itens de edição de texto | Ausentes |
| Ações de imagem | Não existem no registry: o menu Imagem hoje só oferece ferramentas |

---

## 5. Tabela C — Tool rail (19 botões, 5 grupos)

Rail **hardcoded** em `app.slint` §4.1. Cada botão emite `select_tool(<nome>)`
e `main.rs` mapeia para `ToolKind`.

| Grupo | Botão | Ícone | Atalho | `ToolKind` | Status no registry | Situação |
|---|---|---|---|---|---|---|
| 1 Seleção | Select | `MousePointer2` | `V` | `Select` | ✅ | OK |
| 1 | Node | `Pointer` | `A` | `Node` | ✅ | OK |
| 1 | PointTransform | `Move` | `F` | `PointTransform` | ⛔ "folded into the Transform HUD (08.33)" | 🎭 **clicável e ativável**, sem razão visível |
| 1 | Artboard | `Crop` | `H` | `Artboard` | ✅ | OK |
| 2 Desenho | Pen | `PenTool` | `P` | `Pen` | ✅ | OK |
| 2 | Pencil | `Pencil` | `N` | `Pencil` | ✅ | OK |
| 2 | Rectangle | `Square` | `M` | `Rectangle` | ✅ | OK |
| 2 | Ellipse | `Circle` | `E` | `Ellipse` | ✅ | OK |
| 2 | Star | `Star` | `S` | `Star` | ✅ | OK |
| 2 | Polygon | `Hexagon` | `Y` | `Polygon` | ✅ | OK |
| 3 Conteúdo | Text | `Type` | `T` | `ArtisticText` | ✅ | FrameText não tem botão |
| 3 | Gradient | `Palette` | `G` | `Gradient` | ✅ | OK |
| 3 | ColorPicker | `Pipette` | `I` | `ColorPicker` | ✅ | OK |
| 4 Edição vetorial | Corner | `Sparkles` | `C` | `Corner` | ✅ | OK |
| 4 | Knife | `Slice` | `K` | `Knife` | ✅ | OK |
| 4 | Scissors | `Scissors` | `Shift+K` | `Scissors` | **sem entrada** | 🎭 existe na UI e no `ToolKind`, **não existe no registry** |
| 4 | ShapeBuilder | `Combine` | `W` | `ShapeBuilder` | ⛔ "arrives after V1 (10.3 scope)" | 🎭 **clicável e ativável**, sem razão visível |
| 5 Navegação | Hand | `Hand` | `Space` | `Hand` | ✅ | OK |
| 5 | Zoom | `ZoomIn` | `Z` | `Zoom` | ✅ | OK |

**Divisores:** 4 divisores visuais entre grupos.

**Ferramentas registradas como ✅ `Wired` mas SEM botão no rail**
(não alcançáveis por menu nem pela paleta → na prática inalcançáveis):

| Tool | `ToolKind` | Ação |
|---|---|---|
| `ptnd.tool.design.transparency` | `Transparency` | `ptnd.tool.transparency` |
| `ptnd.tool.design.contour` | `Contour` | `ptnd.tool.contour` |
| `ptnd.tool.design.frame_text` | `FrameText` | `ptnd.tool.text.frame` |

**Ferramenta ausente:** `ptnd.tool.design.place_image` (⬜) e
`ptnd.tool.design.line` (⬜), `vector_brush` (⬜).

**Persona Photo: 0 botões.** `ToolKind` tem **9 variantes Photo**
(`MarqueeRect`, `MarqueeEllipse`, `Lasso`, `SelectionBrush`, `FloodSelect`,
`PixelPaintBrush`, `PixelEraser`, `PhotoGradient`, `Crop`) e o registry lista 8
superfícies `ptnd.tool.photo.*`. Nenhuma aparece na UI: o rail é estático e
`active_persona` só altera um texto na status bar.

---

## 6. Tabela D — Todos os `ToolKind` da engine (35) vs. exposição na UI

| # | `ToolKind` | Registry | Botão na UI | Persona |
|---|---|---|---|---|
| 1 | `Select` | ✅ | sim | Design |
| 2 | `Node` | ✅ | sim | Design |
| 3 | `PointTransform` | ⛔ | sim 🎭 | Design |
| 4 | `Pen` | ✅ | sim | Design |
| 5 | `Pencil` | ✅ | sim | Design |
| 6 | `Corner` | ✅ | sim | Design |
| 7 | `Contour` | ✅ | **não** | Design |
| 8 | `Knife` | ✅ | sim | Design |
| 9 | `Scissors` | **sem registro** | sim 🎭 | Design |
| 10 | `Rectangle` | ✅ | sim | Design |
| 11 | `Ellipse` | ✅ | sim | Design |
| 12 | `Polygon` | ✅ | sim | Design |
| 13 | `Star` | ✅ | sim | Design |
| 14 | `ShapeBuilder` | ⛔ | sim 🎭 | Design |
| 15 | `VectorFloodFill` | **sem registro** | **não** | Design |
| 16 | `ArtisticText` | ✅ | sim | Design |
| 17 | `FrameText` | ✅ | **não** | Design |
| 18 | `Gradient` | ✅ | sim | Design |
| 19 | `Transparency` | ✅ | **não** | Design |
| 20 | `ColorPicker` | ✅ | sim | Design |
| 21 | `StylePicker` | **sem registro** | **não** | Design |
| 22 | `Artboard` | ✅ | sim | Design |
| 23 | `Measure` | **sem registro** | **não** | Design |
| 24 | `Zoom` | ✅ | sim | Design |
| 25 | `Hand` | ✅ | sim | Design |
| 26 | `MarqueeRect` | sem registro | **não** | Photo |
| 27 | `MarqueeEllipse` | sem registro | **não** | Photo |
| 28 | `Lasso` | sem registro | **não** | Photo |
| 29 | `SelectionBrush` | sem registro | **não** | Photo |
| 30 | `FloodSelect` | sem registro | **não** | Photo |
| 31 | `PixelPaintBrush` | ⛔ Post-V1 | **não** | Photo |
| 32 | `PixelEraser` | ⛔ Post-V1 | **não** | Photo |
| 33 | `PhotoGradient` | ✅ (`photo.gradient`) | **não** | Photo |
| 34 | `Crop` | ✅ (`photo.crop`) | **não** | Photo |
| 35 | *(Photo move)* | ✅ (`photo.move`) | **não** | Photo |

**Leitura:** a engine tem 35 ferramentas com `action_id` real; a UI expõe 19
botões, todos da persona Design. A dívida é de **exposição**, não de motor.

---

## 7. Tabela E — Dock direito e painéis

Dock direito = **304px**, `panel-tab-height` (32px) + 4px de padding.

| Aba | `active_tab_index` | Painel (`15.G`) | Status | Conteúdo atual |
|---|---|---|---|---|
| **Camadas** | 0 | `ptnd.panel.layers` | ✅ | Ver Tabela F |
| **Propriedades** | 1 | `ptnd.panel.properties` | ✅ | Ver Tabela G |
| **Histórico** | 2 | `ptnd.panel.history` | ✅ | Lista `#N descrição` + botões Undo/Redo |
| ~~Dados~~ | 3 | `ptnd.panel.data_merge` | ⛔ "data merge UI is Post-V1; engine exists" | 🎭 **aba não renderizada**, mas `title_panel_data_merge` é setado em `main.rs:618` (propriedade morta) |

### Painéis declarados no registry

| Painel (`15.G`) | Escopo | Status | Onde existe hoje |
|---|---|---|---|
| `ptnd.panel.layers` | V1 | ✅ | Aba 0 do dock direito |
| `ptnd.panel.properties` | V1 | ✅ | Aba 1 |
| `ptnd.panel.history` | V1 | ✅ | Aba 2 |
| `ptnd.panel.transform` | V1 | ✅ | **Dentro** da aba 1 como grupo "TRANSFORMAÇÃO" — não é painel dockável |
| `ptnd.panel.align` | V1 | ✅ | **Dentro** da aba 1 como grupo "ALINHAMENTO & DISTRIBUIÇÃO" |
| `ptnd.panel.appearance` | V1 | ✅ | **Dentro** da aba 1 como grupo "ESTILO DE PINTURA & GRADIENTES" |
| `ptnd.panel.stroke` | V1 | ✅ | **Dentro** da aba 1 como stack "TRAÇADOS" |
| `ptnd.panel.color` | V1 | ⬜ | **Não existe** |
| `ptnd.panel.swatches` | V1 | ⬜ | **Não existe** (só 7 swatches fixos dentro do inspetor) |
| `ptnd.panel.navigator` | Post-V1 | ⬜ | **Não existe** |
| `ptnd.panel.assets` | Post-V1 | ⬜ | **Não existe** |
| `ptnd.panel.background_tasks` | V1 | ⬜ | **Não existe** |
| `ptnd.panel.data_merge` | Post-V1 | ⛔ | Não renderizado |

**Falta estrutural:** sem docking real, sem splitter, sem arrastar/redimensionar
painéis, sem reordenar abas, sem dock esquerdo, sem dock inferior.

---

## 8. Tabela F — Painel Camadas (item a item)

`app.slint` §4.3, `active_tab_index == 0`.

| # | Item | Posição | Função atual | Falta |
|---|---|---|---|---|
| F1 | Cabeçalho "Layers" + contador `N itens` | topo | Contador vem de `layer_rows.length` | 🌐 título literal por padrão; contador não pluraliza ("1 itens") |
| F2 | Linha "Prancheta Principal" | abaixo do header | 🎭 **texto literal `"📄 Main Artboard [800 × 600]"`** | Dimensões hardcoded — mente se a artboard mudar; sem `TextId`, sem seleção |
| F3 | Toolbar de hierarquia | abaixo de F2 | `Agrupar` / `Desagrupar` / `Máscara` / `Soltar` | 🌐 4 literais; sem habilitar/desabilitar por contexto |
| F4 | Lista de camadas | `ScrollView` 380px | `select_layer_by_index`, badge MÁSCARA/GRUPO, indentação por `depth` | 🌐 "MÁSCARA"/"GRUPO" literais; sem drag & drop; sem rename inline |
| F5 | Toggle de visibilidade | direita da linha | `toggle_layer_visibility(idx)`, `Eye`/`EyeOff` | — |
| F6 | Toggle de trava | direita da linha | `toggle_layer_lock(idx)`, `Lock`/`Unlock` | — |
| F7 | Reordenar ↑/↓ | direita da linha | `reorder_layer_up/down(idx)` | Sem hit-target de 6px; sem arraste |
| F8 | Busca/filtro de camadas | — | — | ⬜ **Não existe** |
| F9 | Botão "adicionar" | — | `add_star_clicked` tem handler em `main.rs:1494` | ⚰️ **Nada emite `add_star_clicked`** — não há como criar estrela pelo painel |
| F10 | Ícone por tipo de camada | esquerda da linha | Mapeia `kind` para 5 ícones | Só `Ellipse`/`Text`/`Star`/container/máscara; demais caem em `Square` |

---

## 9. Tabela G — Inspetor de Propriedades (item a item)

`app.slint` §4.3, `active_tab_index == 1`. 4 grupos em `ScrollView`.

### G.1 Grupo TRANSFORMAÇÃO

| # | Campo | Callback | Falta |
|---|---|---|---|
| G1 | `X (pt)` | `commit_prop_x` | 🌐 rótulo literal; sem unidade selecionável |
| G2 | `Y (pt)` | `commit_prop_y` | idem |
| G3 | `Largura (W)` | `commit_prop_w` | idem |
| G4 | `Altura (H)` | `commit_prop_h` | idem |
| G5 | `Rotação (°)` | `commit_prop_rot` | idem |
| G6 | `Traço (pt)` | `set_stroke_width_value(t.to-float())` | Parse **sem validação** — texto inválido vira `0.0` silenciosamente |

### G.2 Grupo APARÊNCIA (10.4)

| # | Item | Função atual | Falta |
|---|---|---|---|
| G7 | "PREENCHIMENTO (SWATCHES)" + 7 swatches fixos | `set_selected_color(tokens)` | Swatches hardcoded (2 duplicados: `accent-bloom` aparece 2×); sem escolha de cor arbitrária; sem `panel.swatches` real |
| G8 | "ESTILO DE PINTURA & GRADIENTES": `Sólido` / `Grad Linear` / `Grad Radial` | `set_gradient_clicked("solid"\|"linear"\|"radial")` | Sem editor de stops; sem preview |
| G9 | "MODO DE MESCLAGEM" (badge) + 6 botões | `set_blend_mode_clicked(...)` | 6 de 16 modos W3C expostos; 🌐 rótulos literais (Normal/Multiplicar/Tela/Sobrepor/Diferença/Escurecer) |
| G10 | "OPACIDADE GERAL: N%" + `Slider` | `set_opacity_value` | — |
| G11 | "PREENCHIMENTOS (N)" + `+ Preenchimento` | `add_fill_clicked` | 🌐 literal no botão |
| G12 | Stack de fills | `remove_fill_clicked(id)`, mostra `#id [tipo] N%` | Sem editar tipo/opacidade inline; sem reordenar |
| G13 | "TRAÇADOS (N)" + `+ Traçado` | `add_stroke_clicked` | 🌐 literal |
| G14 | Stack de strokes | `remove_stroke_clicked(id)`, mostra `#id (N pt) N%` | Sem editar largura/opacidade inline; sem cap/join/dash |

### G.3 Grupo BOOLEANOS & PATHFINDER (10.3)

| # | Botão | Callback |
|---|---|---|
| G15 | `⋃ Unir` | `boolean_union_clicked` |
| G16 | `− Subtrair` | `boolean_subtract_clicked` |
| G17 | `⋂ Interseção` | `boolean_intersect_clicked` |
| G18 | `⨁ Exclusão` | `boolean_xor_clicked` |

### G.4 Grupo ALINHAMENTO & DISTRIBUIÇÃO (10.1)

| # | Botões | Callback |
|---|---|---|
| G19 | `Esq` / `Centro` / `Dir` | `align_objects_clicked("Left"\|"Center"\|"Right")` |
| G20 | `Topo` / `Meio` / `Base` | `align_objects_clicked("Top"\|"Middle"\|"Bottom")` |
| G21 | `Distribuir H` / `Distribuir V` | `distribute_objects_clicked("Horizontal"\|"Vertical")` |

### G.5 Ausentes do inspetor

| Item | Situação |
|---|---|
| Seção de **Texto** (fonte, corpo, entrelinha, tracking, alinhamento) | ⬜ Ausente — apesar de `ArtisticText`/`FrameText` serem ✅ |
| Seção de **Cor** (modelos sRGB/CMYK/Lab/Spot) | ⬜ Ausente |
| Seção de **Cantos** (raio por canto) | ⬜ Ausente (existe a ação `bake_corners`) |
| Seção de **Export/Prefs por objeto** | ⬜ Ausente |
| Nomes de camada editáveis | ⬜ Ausente |

---

## 10. Tabela H — Context toolbar (item a item)

`app.slint` §3, altura 34px.

> **Estado após o lote `feature/ui-shell-rework` (item 3):** a barra deixou de
> ser uma sequência fixa neste arquivo. A tabela canônica agora é
> `crates/petunia_design_shell/src/context_toolbar.rs` (23 entradas), e a UI
> pinta a lista que recebe. Cada entrada declara seu **escopo** (`Always`,
> `Selection`, `VectorObjects`, `Paths`, `Shapes`): o que não se aplica à
> ferramenta ativa **não chega à UI**, e o que se aplica mas não pode rodar
> aparece desabilitado com o motivo do registry no hover.
>
> Consequências verificadas: os 13 tooltips literais em pt-BR morreram (todo
> rótulo/tooltip vem do catálogo), e o badge passou a exibir o **nome do
> catálogo** ("Caneta", não "Pen"). O id do controle é o que volta para o
> shell, que resolve o **token de menu** equivalente — a mesma lane do menu,
> com o mesmo payload e a mesma disponibilidade.

| # | Entrada (`id`) | Escopo | Posição | Função atual | Falta |
|---|---|---|---|---|---|
| H1 | `ptnd.ctb.tool_badge` | Always | esquerda | Nome do catálogo + glifo; tooltip = nome (atalho) | — (ícone vem do mapa do UI por nome de tool) |
| H2 | `ptnd.ctb.transform` | Selection | esquerda-centro | HUD `X / Y / W / H / R` com largura fixa | Readout puro — **não editável** |
| H3 | `ptnd.ctb.divider.badge` | Always | — | Divisor | — |
| H4 | `ptnd.ctb.swatches` | VectorObjects | centro | 2 swatches com tooltips Preenchimento/Traçado | cor dos swatches não reflete a seleção real |
| H5 | `ptnd.ctb.divider.vector` | VectorObjects | — | Divisor | — |
| H6 | `ptnd.ctb.convert_to_curves` | Paths | centro | ação `object.convert_to_curves` | — |
| H7 | `ptnd.ctb.bake_corners` | Paths | centro | ação `object.bake_corners` | — |
| H8 | `ptnd.ctb.divider.path` | Paths | — | Divisor | — |
| H9 | `ptnd.ctb.boolean.*` (4) | Shapes | centro | ação `object.boolean` com payload `op` | distribuir/máscara/lock ainda fora |
| H10 | `ptnd.ctb.divider.boolean` | Shapes | — | Divisor | — |
| H11 | `ptnd.ctb.align.*` (6) | VectorObjects | centro | ação `object.align` com payload `mode` | distribuir (2) ainda fora |
| H12 | `ptnd.ctb.divider.align` | VectorObjects | — | Divisor | — |
| H13 | `ptnd.ctb.spacer` | Always | — | Espaço flexível (entrada como as demais) | — |
| H14 | `ptnd.ctb.export` | Always | direita | token `file.export#null` (mesma lane do menu, abre o diálogo) | — |
| H15 | `ptnd.ctb.delete` | Always | direita | token `edit.delete#null` | — |

**Ainda falta na barra (declarado, não implementado):**

| Item | Onde está hoje |
|---|---|
| **Personalização** (desativar, adicionar, reordenar, agrupar, divisores) | modelo pronto para receber: as entradas já são uma lista declarada com divisores e spacer de primeira classe; falta a `ToolbarLayout` (ordem + visibilidade) no shell, os métodos no bridge e o popover de customização |
| Distribuir (2), máscara de recorte, lock/hide rápido | existem no registry (menu Objeto/Camada); entram como entradas novas da mesma tabela |
| Opções da ferramenta ativa (ex.: nº de pontas do Star) | não existe modelo de opções de ferramenta no registry |

---

## 11. Tabela I — Status bar (item a item)

`app.slint` §5, altura 26px.

| # | Elemento | Função atual | Falta |
|---|---|---|---|
| I1 | Ícone `Info` | decorativo | — |
| I2 | `status_hint` | Texto por ferramenta (vem de `main.rs`) e por persona | 🌐 **18 literais em português no `main.rs`** (16 hints de ferramenta em 1033–1048 + 2 de persona em 1066/1068) — não passam pelo catálogo |
| I3 | Spacer | — | — |
| I4 | `status_coords` | "Doc: X: N pt  Y: N pt", atualizado no `moved` do canvas | 🎭 formatação literal com `" pt"`; sem unidade selecionável |
| I5 | Seleção + Zoom + Snap | `"1 objeto selecionado"` / `"Nenhum objeto selecionado"` + `%` + Ativo/Inativo | 🎭 **"1 objeto" hardcoded** — não conta a seleção real; 🌐 4 literais |
| I6 | Barra de progresso / tarefas | — | ⬜ Ausente (`panel.background_tasks` é ⬜) |

---

## 12. Tabela J — Diálogos e overlays

| ID (`15.G`) | Tipo | Status | Realidade no código | Falta |
|---|---|---|---|---|
| `ptnd.dialog.new_document` | Dialog | ✅ | ⚠️ **Não existe diálogo.** `new_doc_clicked` → despacha `file.new` direto (`main.rs:1269`) | 🎭 Registry diz `Wired`, mas não há UI de tamanho/sangria/margem. Overstatement |
| `ptnd.dialog.export` | Dialog | ✅ | Overlay Slint real (§5, 2044): PNG/SVG/PDF + descrição + status + Cancelar/Salvar | 🌐 ~12 literais; "Salvar Arquivo com RFD..." expõe a tecnologia; sem opções (DPI, compressão, página) |
| `ptnd.dialog.overwrite_conflict` | Dialog | ⬜ | — | Sem UI de conflito de sobrescrita — `file.save`/`file.export` podem sobrescrever em silêncio |
| `ptnd.dialog.about` | Dialog | ⬜ | — | Não existe |
| `ptnd.window.home` | Window | ⬜ | — | Não existe |
| `ptnd.window.preferences` | Window | ⬜ | — | Não existe |
| `ptnd.window.plugin_manager` | Window | ⬜ | — | Não existe |
| Overlay: command palette | Shell | ✅ | §6 (2208): `LineEdit` + lista de comandos | 🌐 placeholder literal; sem destaque de match; sem navegação por setas na lista |
| Overlay: menu popup | Shell | ✅ | §7 (2289), `StudioMenuPopup` | Sem submenu; sem navegação por teclado (↑↓/Enter) |
| Overlay: confirmação destrutiva | — | ⬜ | — | Deletar/descartar não pedem confirmação |

---

## 13. Tabela K — Atalhos: 41 declarados, 9 ativos

Handler único: `FocusScope` em `app.slint` §0 (linha 341). Não existe tabela de
atalhos global (`slint::Shortcut`) — todo atalho passa por esse `key-pressed`.

### Ativos (9)

| Atalho | Ação | Onde |
|---|---|---|
| `Ctrl+Z` / `Ctrl+Shift+Z` | undo / redo | `FocusScope` |
| `Ctrl+Y` | redo | `FocusScope` |
| `Ctrl+A` | select all | `FocusScope` |
| `Ctrl+D` | duplicate | `FocusScope` |
| `Ctrl+G` / `Ctrl+Shift+G` | group / ungroup | `FocusScope` |
| `Ctrl+K` | command palette | `FocusScope` |
| `Ctrl+E` | export dialog | `FocusScope` |
| `Delete` / `Backspace` | delete | `FocusScope` |
| Setas (+`Shift`=10pt, `Alt`=0.1pt) | nudge | `FocusScope` |
| `Tab` / `Shift+Tab` | cycle selection | `FocusScope` |
| `Esc` | fecha paleta → depois menu | `FocusScope` |

### Declarados no registry mas INERTES (32)

| Grupo | Atalhos | Consequência |
|---|---|---|
| File | `Ctrl+N`, `Ctrl+O`, `Ctrl+S`, `Ctrl+Shift+S` | Salvamento por teclado não funciona; só via menu |
| View | `Ctrl+1`, `Ctrl+0`, `Ctrl+R`, `Ctrl++`, `Ctrl+-` | Zoom e réguas por teclado não funcionam |
| Edit/Clipboard | `Ctrl+C`, `Ctrl+V`, `Ctrl+X` (`PostV1Candidate` ⬜) | — |
| Selection | `Ctrl+Shift+A` | deselect por teclado não funciona |
| **Ferramentas (Design)** | `A B C E F G H I K M N P S T V W Y Z` + `Space` (18 letras + pan) | 🎭 **Nenhum atalho de ferramenta funciona.** Os 19 botões do rail **exibem** `shortcut`, mas nenhuma letra está no `FocusScope` |
| Zoom/Pan | `Space` | Pan por espaço não funciona |

> Consequência de contrato: o `StudioToolButton` **mostra** o atalho ("V", "M",
> "E"…) e o registry o declara. Como nada os escuta, a UI promete o que não
> entrega — o caso mais amplo de 🎭 fake UI deste ledger.

---

## 14. Tabela L — Ações por domínio

### L.1 Despacháveis hoje (65 em `LIVE_ACTIONS`)

**31 ações + 34 ferramentas = 65.**

| Domínio | Ações | Exemplos |
|---|---|---|
| File | **5** | `file.new`, `file.open`, `file.save`, `file.save_as`, `file.export` |
| Edit | **6** | `edit.undo`, `edit.redo`, `edit.duplicate`, `edit.delete`, `edit.select_all`, `edit.deselect` |
| Object | **13** | `object.group`, `object.ungroup`, `object.align`, `object.distribute`, `object.boolean`, `object.arrange.front/back`, `object.clip_mask.create/release`, `object.convert_to_curves`, `object.bake_corners`, `object.lock`, `object.hide` |
| View | **7** | `view.zoom_in/out/100/fit_surface`, `view.toggle_rulers`, `view.toggle_snapping`, `view.command_palette` |
| Tools | **34** | todas as `ptnd.tool.*` com `ToolKind` real |

### L.2 Declaradas mas NÃO ligadas (3) — `DECLARED_NOT_LIVE`

| Ação | Status | Bloqueio real |
|---|---|---|
| `file.place` | ⛔ | `ShapeKind` não tem variante de imagem — não existe onde o asset viver (15.E) |
| `object.offset_path` | ⛔ | distância é escolha do usuário e não há prompt numérico (10.2) |
| `object.slice_path` | ⛔ | precisa de ponto escolhido na path e nenhuma tool fornece (10.2) |

### L.3 Ausentes (9)

`file.open_recent`, `file.close`, `file.quit`, `edit.cut`, `edit.copy`,
`edit.paste`, `edit.preferences`, `select.invert`, `view.focus_canvas`.

### L.5 Ativas na engine mas FORA do registry (9)

Ação resolvível, sem linha de manifesto e sem nenhum controle na UI:

| Ação | `ToolKind` | Persona |
|---|---|---|
| `ptnd.tool.scissors` | `Scissors` | Design |
| `ptnd.tool.measure` | `Measure` | Design |
| `ptnd.tool.style_picker` | `StylePicker` | Design |
| `ptnd.tool.vector_flood_fill` | `VectorFloodFill` | Design |
| `ptnd.tool.photo.marquee_rect` | `MarqueeRect` | Photo |
| `ptnd.tool.photo.marquee_ellipse` | `MarqueeEllipse` | Photo |
| `ptnd.tool.photo.lasso` | `Lasso` | Photo |
| `ptnd.tool.photo.selection_brush` | `SelectionBrush` | Photo |
| `ptnd.tool.photo.flood_select` | `FloodSelect` | Photo |

### L.4 Superfícies `Disabled` (8) — todas com motivo registrado

| Superfície | Motivo |
|---|---|
| `action.file.place` | modelo não tem objeto de imagem |
| `action.object.offset_path` | sem prompt numérico |
| `action.object.slice_path` | sem tool de ponto |
| `tool.design.shape_builder` | Post-V1 |
| `tool.design.point_transform` | dobrado no Transform HUD |
| `tool.photo.brush` | camadas raster são Post-V1 |
| `tool.photo.eraser` | idem |
| `panel.data_merge` | UI Post-V1; engine existe |

---

## 15. Tabela M — Contratos violados hoje (achados verificados)

Ordenados por gravidade. Todos verificáveis com `grep` sobre a árvore atual.

| # | Achado | Evidência | Contrato | Gravidade |
|---|---|---|---|---|
| M1 | **Atalhos de ferramenta prometidos e inertes** — 18 letras declaradas no registry e impressas em cada botão do rail, sem handler | `grep key-pressed app.slint` = 1 ocorrência (linha 341); nenhuma letra avulsa no handler | 15.F §2 "no fake UI" | **Alta** |
| M2 | **Botão de fechar a aba desenhado, callback vazio** | `app.slint:706-707` → `clicked => {}` / `close_clicked => {}`; `StudioTabItem` desenha `IconSet.X` + `TouchArea` | 15.F §2 | **Alta** |
| M3 | **Duas ferramentas `Disabled` no registry são clicáveis e ativáveis na UI**, sem razão visível | Rail: `PointTransform` (⛔ "folded into the Transform HUD") e `ShapeBuilder` (⛔ "arrives after V1") com `clicked => root.select_tool(...)` e handler real em `main.rs` | 15.F §2; `SurfaceStatus::Disabled` | **Alta** |
| M4 | **Três ferramentas ✅ `Wired` são inalcançáveis** — sem botão, sem item de menu, fora da paleta | `ToolKind::Transparency`, `Contour`, `FrameText` existem; nenhum aparece em §4.1 nem em `MENU_BAR` | 15.G "wired ⇒ reachable" | **Alta** |
| M5 | **`dialog.new_document` marcado `Wired` sem existir diálogo** | Registry: ✅ `ptnd.dialog.new_document`; `main.rs:1269` despacha `file.new` direto, sem overlay em `app.slint` | 15.G (status deve refletir a realidade) | **Alta** |
| M6 | **Persona Photo não tem nenhuma ferramenta na UI** | 9 `ToolKind` Photo + 8 superfícies `ptnd.tool.photo.*`; `active_persona` só troca o hint de status (`main.rs:1062`) | 15.F §5/§6 (workspaces por persona) | **Alta** |
| M7 | **"1 objeto selecionado" hardcoded** | `app.slint` §5: string literal, não conta a seleção | 15.F §2 | Média |
| M8 | **Linha da prancheta com dimensões hardcoded** | `app.slint:1510`: `"📄 Main Artboard [800 × 600]"` literal | 15.F §2 | Média |
| M9 | **Indicador "modificado" sempre aceso** | `app.slint:704` → `is_dirty: true` fixo | 15.F §2 | Média |
| M10 | **Swatches de Fill/Stroke da toolbar são cor fixa** | §3: `set_selected_color(Tokens.accent-bloom)` — não lê a seleção | 15.F §2 | Média |
| M11 | **`add_star_clicked` tem handler e nenhum emissor** | `main.rs:1494` + `app.slint:526` declarado; nenhum `root.add_star_clicked()` | ⚰️ código morto | Baixa |
| M12 | **`open_doc_clicked` / `save_doc_clicked` / `place_image_clicked` sem emissor na UI** | Handler existe; nada emite | ⚰️ código morto | Baixa |
| M13 | **`title_panel_data_merge` é setado para uma aba que não existe** | `main.rs:618` + propriedade em `app.slint:581`; `active_tab_index` nunca chega a 3 | ⚰️ propriedade morta | Baixa |
| M14 | **9 ferramentas ativas na engine não têm linha no registry** — 1 delas (`Scissors`) tem botão no rail | `LIVE_ACTIONS` contém `ptnd.tool.scissors`, `measure`, `style_picker`, `vector_flood_fill`, `photo.marquee_rect`, `photo.marquee_ellipse`, `photo.lasso`, `photo.selection_brush`, `photo.flood_select`; nenhuma consta em `SURFACES` | 15.G (toda superfície no manifest) | Média |
| M15 | **Parse de campo numérico sem validação** | `set_stroke_width_value(t.to-float())` → entrada inválida vira `0.0` | — | Média |
| M16 | **79 strings user-facing literais em `app.slint`** | `grep -c 'text: "'` = 79 | 09.16 i18n | Média |
| M17 | **18 hints de status literais em `main.rs`** | 16 em `1033–1048` (mapa ferramenta→texto) + 2 em `1066/1068` (persona) | 09.16 i18n | Média |
| M18 | **Ferramentas não são alcançáveis por menu nem pela paleta** | `MENU_BAR` tem 6 famílias, nenhuma de ferramentas; `command_index()` deriva do menu | 15.F §3 | Média |
| M19 | **Sem confirmação para ações destrutivas** | `dialog.overwrite_conflict` ⬜; delete sem confirmação | — | Média |
| M20 | **Contador "N itens" não pluraliza** | `layer_rows.length + " itens"` | — | Baixa |

---

## 16. Ordem de trabalho sugerida (grupos de PR)

Cada grupo é fechável de forma independente e verificável.

### Onda 1 — Parar de mentir (correções de contrato)
Baixo esforço, alto impacto de confiança. Nenhuma feature nova.

| PR | Escopo | IDs |
|---|---|---|
| 1.1 | Ligar os atalhos de ferramenta no `FocusScope` | M1 |
| 1.2 | Ligar/remover o X da aba; corrigir `is_dirty` | M2, M9 |
| 1.3 | Bloquear de verdade `PointTransform` e `ShapeBuilder` (ou desabilitar com motivo) | M3 |
| 1.4 | Corrigir status do registry onde ele exagera (`dialog.new_document`) | M5 |
| 1.5 | Registrar as 9 ferramentas ativas sem linha no registry; corrigir ações órfãs | M14, M11, M12, M13 |
| 1.6 | Remover os readouts falsos (objeto selecionado, dimensões, swatches) | M7, M8, M10 |

### Onda 2 — i18n de chrome
O encanamento já existe (`shell_strings.rs` + `LocalizationService::with_shell_catalog`).

| PR | Escopo | IDs |
|---|---|---|
| 2.1 | Substituir **os 79 literais** de `app.slint` por `TextId` alimentados no `sync_ui_from_shell` | M16 |
| 2.2 | Mover os 18 hints de `main.rs` para o catálogo | M17 |
| 2.3 | Traduzir o export dialog + labels de ferramenta (nome canônico, não o id interno) | H1, J2 |

### Onda 3 — Tornar alcançável o que já funciona
Nenhuma linha de engine; só exposição.

| PR | Escopo | IDs |
|---|---|---|
| 3.1 | Expor `Transparency`, `Contour`, `FrameText` no rail (ou num grupo "mais") | M4 |
| 3.2 | Família de menu **Tool** + entrada na paleta | M18 |
| 3.3 | Rail por persona: implementar o conjunto Photo | M6 |
| 3.4 | Botão "+ Estrela" no painel de camadas | M11 |

### Onda 4 — Itens de UI ausentes (um por PR)

| PR | Item | Dependência |
|---|---|---|
| 4.1 | `panel.color` (seletor de cor com modelos sRGB/CMYK/Lab/Spot) | — |
| 4.2 | `panel.swatches` (bibliotecas de cor) | 4.1 |
| 4.3 | Seção de **Texto** no inspetor | — |
| 4.4 | Seção de **Cantos** (raio por canto) | — |
| 4.5 | `dialog.overwrite_conflict` + confirmação destrutiva | — |
| 4.6 | Busca/filtro de camadas + rename inline | — |
| 4.7 | `dialog.about` | — |
| 4.8 | `panel.background_tasks` | — |
| 4.9 | `surface.shell.overflow` (menu "…" responsivo) | — |
| 4.10 | Overlay de progresso de export | — |

### Onda 5 — Estrutura

| PR | Item | Dependência |
|---|---|---|
| 5.1 | Splitter real (1px visível / 6px hit) + resize do dock direito | — |
| 5.2 | Docking: arrastar/reordenar/destravar abas | 5.1 |
| 5.3 | `dock.left` | 5.2 |
| 5.4 | `dock.bottom` | 5.2 |
| 5.5 | `panel.navigator` | 5.4 |
| 5.6 | `panel.assets` | 5.2 |
| 5.7 | `file.place` — variante de imagem no `ShapeKind` + importer + nó de render | Onda engine |

### Onda 6 — Qualidade

| PR | Item |
|---|---|
| 6.1 | Acessibilidade `08.36` (roles, foco, ordem de tabulação, contraste) |
| 6.2 | QA visual Slint de `15.F` com a app rodando |
| 6.3 | Performance / RAM / VRAM (`15.H`) |
| 6.4 | Security corpus / fuzz (`15.D`) |

---

## 17. Verificação (rodar a cada PR)

```bash
# Gate completo: fmt --check + clippy -D warnings + testes + arquitetura
cargo run -p xtask -- verify

# Gauntlet P00: testes + arch + CLI conformance + fixtures + docs
cargo run -p xtask -- gauntlet

# Só o app Slint (força rebuild do .slint)
cargo test -p petunia-design
```

**Testes de contrato do app** (`main.rs`, `mod tests`) — estenda-os quando
tocar tokens/UI:

| Teste | O que garante |
|---|---|
| `ui_references_only_declared_tokens` | Nenhum token fantasma em `app.slint` |
| `ui_color_literals_are_confined_to_document_artwork` | Hex cru só no artwork (08.21) |
| `ui_shell_rows_use_their_canonical_geometry_tokens` | Rows ambíguos usam o token nomeado |

**Testes de contrato do shell** (`crates/petunia_design_shell/tests/menu_test.rs`):

| Teste | O que garante |
|---|---|
| `every_wired_action_is_reachable` | Ação `Wired` sem caminho falha |
| `command_palette_offers_exactly_the_enabled_menu_items` | Paleta não diverge do menu |

**Testes do registry** (`surfaces.rs`, `mod tests`):

| Teste | O que garante |
|---|---|
| `disabled_surfaces_explain_themselves` | Todo `Disabled` tem motivo com >12 chars |
| reconciliação `DECLARED_NOT_LIVE` | Declarado-e-não-ligado consta na `SURFACES` e **não** consta em `LIVE_ACTIONS` |

> Para os itens M1–M20 **não existe teste hoje**. Cada PR da Onda 1 deve trazer
> um teste que falhe antes e passe depois — senão a correção volta.

---

## 18. Invariantes por item (checar antes de fechar qualquer PR)

1. **Uma ação, uma lane.** UI → `ActionRequest` → `Command` → `DocumentMutator` → `ChangeSet`. `.slint` nunca toca armazenamento.
2. **Crates de domínio não importam toolkit de UI.** `xtask arch` é o juiz.
3. **Sem UI falsa.** Botão sem ação, `todo!()`, callback vazio ou placeholder são proibidos. Ausente = implementar, desabilitar com motivo, esconder ou marcar experimental.
4. **Toda superfície tem ID estável** no registry, com `kind`, `scope`, `status`, `label`, `action`, `shortcut`. `Vec` index não é identidade.
5. **`Disabled` sempre carrega motivo** legível pelo usuário.
6. **`Wired` exige caminho de resolução** — constante em `actions.rs` é declaração, não implementação.
7. **Slint é a única UI.** egui/gpui/iced/desktop estão aposentados.
8. **`aubrieta.*` só é lido, nunca emitido.**
9. **Compilação ≠ implementação. Teste pulado ≠ PASS.**
10. **Geometria literal só onde `tokens.slint` declara a política** (documento, janela/diálogo, micro-geometria sem métrica canônica). Inventar token para o resto é fabricar contrato.

---

*Gerado a partir da árvore de trabalho sobre `e90a6c9`, branch
`refactor/petunia-design-studio`, com alterações não commitadas. Revalide os
números de §1 no registry antes de citá-los — eles mudam a cada superfície
ligada ou desligada.*
