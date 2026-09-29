# Diário de Decisões — Ferramentas (09.31 na prática)

Log append-only. Cada batch registra: decisão, referência de mercado, porquê.
Doutrina-mãe: ADR 09.31 (`petunia-design-studio/09 31 — Non-Destructive Editing EffectChain ADR.md`).

## Doutrina operacional (vale para todas as ferramentas)

- **Edição de base vs. modificador.** Criar traço, mover nó, deletar nó,
  fatiar path: edições topológicas da fonte — a fonte É o path, nada se
  preserva além dela, undo cobre. Transformação paramétrica (offset, cantos
  vivos, warp futuro): modificador vivo, fonte intacta, Bake explícito.
- **Um gesto, um undo (F-01).** `Move` prevê, `Up` commita via `submit_all`.
  Sem escrita por evento.
- **Conversão implícita só dentro do gesto que a exige.** Faca corta qualquer
  forma: `ConvertToCurves` entra no mesmo batch do corte (o gesto pede corte,
  não existe corte sem curva). Fora de gesto, conversão continua explícita.
- **Stubs são silenciosos por construção antiga;** todo stub tocado ganha
  comportamento ou registro de futuro — nunca `todo!()`.

## Roadmap decidido (ordem de execução)

| # | Batch | Escopo | Motivo da ordem |
|---|---|---|---|
| 6 | Knife × Scissors | Linha de corte amostrada em 1 undo; tesoura por clique; auto-convert no batch | Pequeno, fecha a família 10.2 |
| 7 | Gradient | Stops múltiplos, radial, snap de ângulo, preview vivo | Médio, sem modelo novo |

## Batch 7 — Gradient (decisões)

- **Linear + Radial, sem cônico.** `Paint` não tem `ConicGradient` e SVG/PDF não
  o representam — modelo novo + fallback de export é outro batch. Registrado futuro.
- **Stops no canvas:** duplo-clique na linha adiciona (cor amostrada do ponto
  médio em literal `rgb()`), duplo-clique no stop remove (mínimo 2), arrastar
  move offset (Shift = passos de 0.05). Overlay `GradientOverlay` novo expõe
  linha + alças (aditivo em `CanvasOverlays`, sem quebrar leitores).
- **Sem cor inventada:** criação a partir de sólido duplica a cor nos 2 stops;
  troca de geometria preserva stops; `sort_and_reindex` mantém ids estáveis.
- **Clique nunca cria** (limiar 3 px). Shift trava vetor em 45°. Um gesto, um undo.
- **Transparency intocado:** continua proxy documentado até virar modificador (batch 10).
| 8 | Picker + Measure | Amostra stroke/gradiente/appearance; Measure com área | Pequenos, mesmo padrão |

## Batch 8 — Picker + Measure (decisões)

- **Color = par completo.** Amostra fill primário (sólido ou gradiente preservado)
  + stroke (paint + largura) e aplica via um `SetAppearance` por alvo: preserva
  resto da stack, 1 undo. Style continua stack inteira. Sem split por modificador
  (previsível > esperto).
- **Travados nunca amostram** (filtro `visible && !locked` — faltava).
- **Measure com área:** `MeasureMode::{Distance, Area}`; arrasto = retângulo
  (área + perímetro, overlay marquee); parado com seleção = soma das áreas
  avaliadas (`measured_area`, respeita modificadores vivos).
- **Infra:** `AreaReadout`/`area_readout` na geometria (Table B) para a UI ler.

## Nota de infra (descoberta nos batches 7–8)

- O `target-dir` compartilhado entre worktrees envenena fingerprints quando dois
  agentes compilam juntos (sintomas: itens "inexistentes" que existem).
  **Regra: `CARGO_TARGET_DIR=/tmp/petunia-tools-target` em todo comando cargo
  desta worktree.**
| 9 | ShapeBuilder | Síntese booleana de região com highlight + 1 undo | Médio, usa boolean avaliado |

## Batch 9 — ShapeBuilder + SmartFill (decisões)

- **Região = face por assinatura de cobertura** (interseção de quem cobre menos
  união do resto), sobre outlines avaliados. Para 2 formas = faces exatas do
  Illustrator; para N, aproximação documentada (faces distintas podem partilhar
  assinatura). Flood de espaço negativo aberto fica futuro (face detection).
- **Click cria, Alt subtrai, drag funde.** Subtração que consome tudo deleta o
  objeto; fontes do merge são preservadas (desvio documentado do AI, que
  consome — reversível via undo, sem perda silenciosa).
- **SmartFill = mesmo motor, fill default** (`ptnd.blue/500`, precedente do
  gradient). Flood real fica futuro.
- **Achado:** união booleana com acumulador vazio retornava vazio para sempre
  (identidade quebrada) — corrigido com teste.
- **Overlay:** `region_preview` (doc-space) para highlight de hover/drag.
| 10 | Transparency como modificador | `TransparentGradient` vira 2º `ModifierKind` (gatilho de revisita do ADR: generaliza identidade/ordem) | Médio-grande, valida a fundação |

## Batch 10 — TransparentGradient, 2º modificador (decisões)

- **Modelo:** `TransparentGradient { start, end, stops: OpacityStop[] }`;
  avaliação no domínio opacidade (`evaluate_opacity_at`, produto das entradas),
  geometria intacta. Domínios independentes: contour e transparency coexistem
  sem vazar um no outro.
- **Ferramenta:** arrasto define o vetor (Shift 45°), default opaque→transparent;
  substitui o proxy `SetStackOpacity`. Clique nunca cria.
- **Render honesto:** sem infra de máscara, export/preview amostram o centro
  (`sampled_opacity`, documentado); máscara cheia fica futuro.
- **Bake explícito:** `bake_transparency` achata a amostra central na opacidade
  base (documentado com perda) + `bake_contour` já existia.
- **API de cadeia (futura UI de lista):** enable/disable/remove/move-to-front via
  `SetModifiers`, NoOp sem entrada no histórico (F-22).
- **Check de revisita do ADR 09.31:** generalização OK (identidade por `id`,
  ordem = ordem do vec, custo linear); falta só UI de lista/ordem.
| 11 | Text-on-Path | Handles start/end, fluxo no path | Médio, spec 10.6 V1 |

## Batch 11 — Text-on-Path (decisões)

- **Modelo:** `ShapeKind::Text.on_path: Option<TextOnPathAttachment { target, start, end }>`
  (`serde default`: arquivos v1 abrem). Sem `ToolKind` novo — a Text tool cria ao
  clicar/arrastar sobre um path (Illustrator). O path nunca é consumido.
- **Geometria:** `GPath::{outline_length, sample_at, nearest_t}` por comprimento
  de arco achatado (F-21; exato-em-curva POST_V1). Span degenerado ganha mínimo 0.01.
- **Gestos:** clique cria start=t..1.0; drag define start..end; handles arrastam
  com 1 undo; Alt-clique/detach volta a reto. Bounds do texto = span.
- **Export honesto:** SVG emite `<text><textPath href="#target">` (referência por id,
  path d embutido em comentário); PDF registra `TEXT_ON_PATH_FLATTENED` e cai no
  fluxo de bounds. Preview curvo de glifos (shaping) fica futuro.
- **Regressão:** clique longe do path continua criando headline 160×32.

## Batch 12 — SmartFill flood de espaço negativo (decisões)

- **Algoritmo:** `frame − união(obstáculos)` num booleano só; a face que contém
  o clique (menor área, pois furos são irmãos flat) vira o objeto. Faces que
  tocam o frame = ilimitadas = NoOp (encher o infinito criaria lixo).
- **Strokes abertos delimitam:** centerlines viram bandas pelo `offset_path`
  (meia largura do stroke) antes do booleano — mesma matemática do Contour.
- **Escopo:** seleção quando não-vazia, senão tudo visível/destravado.
- **Achado (bug real no batch 9):** `difference_many` par-a-par quebrava
  semântica de furos; unificado numa chamada (furos são contornos irmãos,
  cada passo precisa ver a forma inteira). Vale para subtração de região toda.
- **SmartFill drag** inunda no release (semântica de clique); Alt sobre vazio = NoOp.
| 12 | SmartFill flood real | Face de espaço negativo via frame−união | Feito batch 12 |
| 13 | Seleção raster (epic) | Máscara de contornos + marquee/lasso/modos | Feito batch 13 (flood/brush futuros) |

## Batch 13 — Seleção raster: máscara de contornos (decisões)

- **Modelo:** `RasterSelection { contours, feather }` em `application::selection_mask`,
  estado transiente de sessão (como seleção de objetos: sem undo, sem ChangeSet).
  Contenção even-odd sobre contornos flat — mesma convenção de furos dos booleanos.
- **Motor:** marquee rect/ellipse/lasso commitam de verdade; modos via Shift/Alt
  (Replace/Add/Subtract/Intersect, convenção Photoshop); clique limpa (Replace).
  Invert dentro dos bounds da surface; Grow/Shrink pelo offset real (Miter);
  feather é parâmetro de render, nunca geometria.
- **Overlay:** `selection_mask` (doc-space) alimenta marching ants futuras.
- **Fora (futuros documentados, exigem pixel-layers no documento):**
  FloodSelect/SelectionBrush (amostragem de pixels), Brush/Eraser (pintura),
  feather renderizado, Refine/QuickMask/Straighten.
| 14 | Warp/Perspective (epic) | 3º modificador + tool, 10.8 V1_REQUIRED | Feito batch 14 (envelope futuro) |

## Batch 14 — Perspective + Crop vetorial (decisões)

- **Perspective = 3º `ModifierKind`** (`quad` TL/TR/BR/BL absoluto). Avaliação por
  homografia DLT sobre o bbox base: identidade preserva curvas, warp real
  achata em 0.25pt (F-21, mesma doutrina do inset); quads degenerados e
  cruzamentos de vanishing (`w≈0`) recusam e preservam o anterior.
- **ToolKind::Perspective** (`ptnd.tool.perspective`, registry Wired + LIVE_ACTIONS,
  botão BoxSelect no rail): arrasta 1 canto por vez, 1 undo para a seleção toda;
  quads absolutos (mover o objeto depois não move o warp — limite V1).
- **Crop vetorial = 4º modificador** (`CropRect`): interseção para fechados,
  Liang–Barsky por runs para abertos (strokes aparados, não somem); vazio
  preserva. Crop tool com seleção = crop vetorial; sem seleção = surface crop.
- **BakeGeometry** commita contour+perspective+crop e preserva transparency.
- **Check de revisita do ADR 09.31:** 4 kinds, identidade por `id`, ordem = vec,
  custo linear — fundação validada; falta só UI de lista/ordem.

## Avulsos V1 — decisões finais

- **Place Image (epic, V1Required mas Absent):** o picker (`on_place_image_clicked`)
  cria objeto SEM pixels (hollow). Falta `ShapeKind::Image` + armazenamento de
  recurso + render/export + `file.place` (`DECLARED_NOT_LIVE`). Design: shape com
  `resource: ResourceId` + dimensões, `import_raster` já decodifica — epic próprio.
- **Line:** registry `PostV1Candidate/Absent` — confirmado futuro, sem ToolKind.
- **Stroke Width tool:** exige perfil de largura no modelo (`StrokeItem.width` é
  escalar) — futuro com Vector Brush, mesma fundação.
- **Vector Brush:** Post-V1 por escopo (08.24/08.33) — confirmado, sem ToolKind.
- **Envelope mesh / grid warp (10.8 resto):** futuro sobre a mesma `warp_path`
  (subdividir + mapear pontos de controle); `Homography` reutilizável.
- **Registry `shape_builder: Disabled`:** divergente (batch 9 shipou) — nota para
  o agente de UI; esta worktree não muda status alheio.
| — | Photo paint/retouch, Vector Brush, Place Image, Vector Crop, Stroke Width, Line | Batches avulsos por demanda | Pequenos-médios |

## Batch 6 — Knife × Scissors (decisões)

- **Knife = linha de corte (Corel/Affinity).** A linha do arrasto vira peças de
  verdade via `geometry::cut_path_by_line` (Sutherland–Hodgman por semiplano,
  fechado fica fechado), tudo num único `submit_all`. Primeira tentativa
  (amostrar a linha e furar pontos) foi descartada: `slice_path` só move a
  costura, corte invisível.
- **Scissors = clique que parte (Illustrator).** Down+Up sem arrasto =
  `split_path_at_point`: loop abre em 1 peça, stroke aberto divide em 2.
- **Alvos:** proximidade ao contorno avaliado + `hit_test`, topmost-first,
  visível/destravado; paramétricos auto-convertem no batch, texto e
  containers excluídos (doutrina: conversão dentro do gesto).
- **Achado registrado:** `hit_test` nunca acerta path aberto (sem interior) —
  Select também não clica strokes abertos. Fix global de hit stroke-aware fica
  futuro; a faca resolve no escopo com `near_object`.
- **Sem fishermen:** peças degeneradas (<1pt de comprimento) caem; sem remoção
  de micro-fragmentos (10.2: sem deleção silenciosa).
- **Overlay distinto:** Knife mostra a linha; Scissors mostra o ponto de corte.

## Wave 0 — Canvas, input e ordem de implementação (2026-09-25)

- **Shell ativo:** Freya é o adaptador de produto em desenvolvimento nesta worktree. Slint fica congelado como referência histórica de rendering/painéis até a autorização de uma decisão de shell própria.
- **Fonte única de overlays:** `ToolManager::overlays` resolve a ferramenta ativa; `PetuniaShell::overlays` acrescenta as guias de snap. A política `workspace_overlays` paralela foi removida para impedir que o canvas Freya descarte previews de Corner, Contour, Perspective, ShapeBuilder, texto, cut tools e Photo.
- **Input:** o adaptador Freya preserva `PointerButton` Left/Middle/Right, trata toque sem botão como primário e não converte botões desconhecidos em primário. `Move` mantém a semântica de ponteiro primário atual; captura/foco de ponteiro é uma decisão posterior.
- **Documento canônico:** a matriz detalhada das 35 ferramentas está em `petunia-design-studio/13 — Full Notebook Page-by-Page Audit & Conformance Ledger`; este diário registra somente decisões e dependências.
- **Ordem de implementação:** Select/Transform/Canvas → Pen/Node/Corner/Contour → Shapes/Boolean/ShapeBuilder/SmartFill → Fill/Stroke/Gradient/Transparency → Text/Text-on-Path → Photo/raster → Perspective/Warp/Grid → Place Image/painéis.
- **Regra de evidência:** nenhum status `Proven` sem teste headless, contrato Action/Command/ChangeSet, persistência/undo, UI semântica, acessibilidade e benchmark aplicáveis.
- **Estado atual da Wave 0:** a suíte completa `tools_test` passou com **102 testes**; o crate `petunia_design_document` passou com **43 testes** (39 unit + 4 property); a regressão estrutural `canvas_snapshot_uses_world_frame_and_rotation` passou; `cargo check -p petunia_design_shell -p petunia-design --bin petunia-design` passou. O DTO `CanvasSnapshot` e o frame mundial estão conectados, mas a paridade visual completa e a medição release ainda permanecem abertas.
- **Select + Transform — contrato entregue:** `TransformPreview` é um DTO toolkit-neutral em `petunia_design_shell::canvas`, com `base_revision`, frame AABB e propostas por `ObjectId`. `SelectTool` captura a revisão no `Down`, publica somente durante arraste efetivo, bloqueia commit obsoleto, limpa preview no `Up/Cancel` e rejeita preview cuja revisão já foi substituída. A matemática de rotação usa o pivô do centro da seleção para mover e somar rotação a cada objeto; o commit e o preview compartilham o mesmo cálculo.
- **Canvas Freya — slice atual:** o renderer desenha a proposta como contorno translúcido usando elementos nativos do Freya, sem SVG por objeto, e aplica a rotação no pivô superior-esquerdo exigido pelo modelo `T(bounds_origin) * R`. O adapter consome `CanvasSnapshot` com frame/transform/AABB mundiais, cache, culling e overlays completos; isto prova a integração do contrato, não equivalência visual com Affinity, Krita, Figma ou Inkscape.
- **Limite explícito da fatia:** `CanvasObjectProjection` agora expõe frame/transform/AABB mundiais, mas o renderer ainda representa a cena visual apenas por bounds/fill/shape. Paths Bézier, texto, strokes, gradientes, imagens, efeitos, masks complexas e handles de editação geométrica não têm paridade visual comprovada. Não classificar `Proven` nem escolher Vello/wgpu até fixtures headless/release medirem hover, marquee, move/resize/rotate, snapshot e input-to-frame.
- **Evidência de regressão:** 14 testes `select_` passaram, incluindo publicação/limpeza de preview, cancelamento, commit obsoleto, snapshot obsoleto e rotação multi-objeto; a suíte completa `tools_test` passou com 102 testes. O crate document passou com 43 testes e a regressão estrutural do snapshot mundial passou. Esses números validam comportamento headless, integração do shell e projeção estrutural, não performance de produção nem paridade visual.

## Wave 1 — Ciclos 5 a 8: Feedback Visual, Navegação Fluida, Context Toolbar e Tabs (2026-09-26)

- **Ciclo 5 (Canvas Overlays & Visual Feedback):**
  - **Marching Ants:** Implementado algoritmo determinístico de traço pontilhado alternado preto/branco (`paint_dashed_line`, `paint_marching_ants_rect`, `paint_marching_ants_polyline`) sem dependência de bindings externas de `PathEffect` do Skia, garantindo visualização precisa para seleções raster (`marquee_screen` e contornos de `selection_mask`).
  - **Grid de Perspectiva 3x3:** Adicionado wireframe interno 3x3 com subdivisões proporcionais interpoladas bilinearmente dentro dos 4 vértices do quad no `PerspectiveOverlay`.
  - **Guias de Superfície Persistentes:** O snapshot de visualização (`SurfaceView`) agora transporta as guias persistentes da superfície ativa (`guides: Vec<petunia_design_document::Guide>`), renderizadas em ciano pontilhado (`0x00, 0xBC, 0xD4`).
- **Ciclo 6 (Navegação Fluida & Criação de Guias pelas Réguas):**
  - **Navegação Contínua com Botão do Meio:** O viewport agora detecta `PointerButton::Middle` no `on_pointer_down` e permite translação instantânea sem latência durante o arraste.
  - **Zoom Centrado no Cursor:** O evento de roda (`on_wheel`) com tecla modificadora Ctrl/Cmd ativa utiliza `shell.zoom_at(screen_focus, factor)` preservando o ponto sob o cursor estável no espaço do documento. Rolar sem modificador realiza pan suave horizontal/vertical.
  - **Criação de Guias por Arraste das Réguas:** Clicar e arrastar a partir da régua superior (`y < 20.0, x >= 20.0`) ou da régua esquerda (`x < 20.0, y >= 20.0`) inicia um drag de guia com badge flutuante de coordenadas em tempo real. No `PointerUp`, um `Command::AddGuide` canônico é despachado para a superfície ativa.
- **Ciclo 7 (Barra de Ferramentas de Contexto Dinâmica):**
  - Controles rápidos e botões de ação contextuais na barra de contexto (`chrome.rs`) baseados na ferramenta ativa (`ToolKind`):
    - `Rectangle` / `Corner`: Ações "Fixar Cantos" (`ptnd.action.object.bake_corners`) e "Para Curvas" (`ptnd.action.object.convert_to_curves`).
    - `Pen` / `Node`: Ação "Converter em Curvas" (`ptnd.action.object.convert_to_curves`).
    - `Select`: Botões de operações booleanas imediatas ("União", "Subtrair", "Interseção").
- **Ciclo 8 (Tab Strip Multi-Documento e Toggles de Visualização):**
  - Tab strip real com nome do documento aberto, indicador dirty circular âmbar (`is_dirty`), botão fechar aba (`×`), e botão de nova aba (`+` chamando `ptnd.action.file.new`).
  - Acesso rápido a toggles de visualização no canto direito do tab strip: Snap liga/desliga (`ptnd.surface.tabs.snapping`), Réguas liga/desliga (`ptnd.action.view.toggle_rulers`) e Enquadrar na Janela (`ptnd.action.view.fit_surface`).
- **Validação e Integridade:**
  - Todos os 18 testes de `petunia-design` passaram, incluindo o novo teste unitário de criação de guias por arraste (`ruler_drag_creates_horizontal_and_vertical_guides`).
  - `cargo test --workspace` (todos os 117 testes de ferramentas, 107 testes de aplicação, testes de viewport e proptests) passou 100%.
  - `cargo check --workspace` passou com zero warnings e zero erros.

## Wave 2 — Ciclos 1 a 5: Place Image, Edição de Texto In-Canvas, Diálogos do Sistema, Dock Splitter / Minimap e Salvaguardas de Fechamento (2026-09-26)

- **Item 1: Place Image (`ptnd.action.file.place`):**
  - Integração ponta-a-ponta da importação de imagens raster: ação migrada de `DECLARED_NOT_LIVE` para `LIVE_ACTIONS` com status `SurfaceStatus::Wired`.
  - No handler de sessão (`session.rs`), a carga do arquivo é decodificada via `petunia_design_io::import_raster`, calculando largura e altura reais e anexando os bytes em `data: Some(bytes)`.
  - Criado objeto de formato `ShapeKind::Image { path, data }`, renderizado no canvas Skia como bitmap decodificado (`skia_safe::Image::from_encoded`).
- **Item 2: Edição de Texto In-Canvas:**
  - Editor flutuante in-canvas renderizado diretamente sobre a posição de tela do objeto `ShapeKind::Text` ativo selecionado (`screen_origin` calculado pela projeção de câmera).
  - Inclui input de texto reativo e botão "Aplicar", despachando `Command::SetShape` na lane de comandos formal, preservando família tipográfica, tamanho da fonte e alinhamentos.
- **Item 3: Diálogos do Sistema (`NewDocumentDialog` e `ExportDialog`):**
  - `NewDocumentDialog`: Permite selecionar presets de artboard ("Web 1080p", "Quadrado 1000", "Mobile 390x844", "A4 Print") ou especificar dimensões customizadas em pixels, despachando `Command::SetSurfaceGeometry`.
  - `ExportDialog`: Permite selecionar o formato de destino (PNG bitmap, SVG vetorial, PDF documento) e caminho do arquivo, despachando `ptnd.action.file.export`.
  - Acessíveis via Menu Superior (`File > New`, `File > Export`), atalhos globais (`Ctrl+N`, `Ctrl+E`) e Command Palette (`Ctrl+K`).
- **Item 4: Dock Splitter & Minimap/Navigator:**
  - `DockSplitter`: Divisor arrastável interativo de 4px entre Workspace e RightDock com feedback visual (cor de destaque ao arrastar e cursor `EwResize`). Utiliza um overlay de captura via `Portal` durante o drag para garantir rastreamento contínuo e clamping de largura entre 180px e 520px.
  - Aba "Navegador" (5ª aba no RightDock): Exibe nível de zoom atual em %, coordenadas de pan (X, Y), botões de zoom rápido (-25%, +25%, Enquadrar) e um minimap proporcional da área de trabalho e artboard.
- **Item 5: Salvaguarda de Fechamento (`ConfirmCloseDialog`):**
  - Diálogo modal de confirmação para prevenir perda de dados acidental: acionado ao clicar em fechar aba (`×`) quando o documento possui alterações não salvas (`shell.bridge.is_dirty()`).
  - Permite ao usuário cancelar ou confirmar "Fechar Sem Salvar", invocando o fechamento da sessão de forma segura.
- **Evidências de Teste e Qualidade:**
  - Suíte `petunia-design` ampliada para 24 testes (14 testes unitários de workspace/dock + 10 testes de chrome), todos com 100% de sucesso.
  - `cargo test --workspace` 100% verde em todos os crates da aplicação, shell, geometria, documentos e testkit.
  - `cargo check --workspace` com zero warnings e zero erros.

## Decisão — Shell Freya, sem fork (2026-09-29)

- **Decisão:** ficar no Freya upstream pinado (`freya 0.5.0-rc.7`, `Cargo.lock`
  travado, binary-cache do Skia). **Sem fork** — nem hard, nem soft — até um
  gatilho abaixo disparar.
- **Porquê:** os 4 gargalos abertos (snapshot O(n) F6, texto real F7, raster
  paint, tetos de modelo) moram no nosso código, não no framework. Fork herdaria
  ~93k SLoC de binding Skia + upgrades `winit` + bugs por plataforma
  (GL/Vulkan/softbuffer) e desviaria ~20–30% do time do editor para manter GUI,
  sem acelerar nenhum item do caminho crítico.
- **Fronteira que dispensa o fork:** `PetuniaShell` + `CanvasSnapshot`/`CanvasOverlays`
  (DTOs toolkit-neutral em `petunia_design_shell::canvas`) já isolam a cena.
  A UI só recebe projeções e só emite `ActionRequest`/`Command` — trocar ou
  forkar o renderer continua possível sem tocar o domínio.
- **Exit strategy (`SceneRenderer`):** quando a F6 começar, extrair trait
  `SceneRenderer` atrás de `canvas_paint.rs` (snapshot → pintura Skia), para que
  um backend alternativo (egui-textura, `PaintCallback` nativo, Vello futuro)
  seja plugável por medição, não por reescrita.
- **Gatilhos para soft-fork (`[patch]`/fork rastreado, nunca hard de imediato):**
  1. Freya bloqueia 2 releases seguidos nossos; 2. PR com necessidade nossa fica
  sem resposta >60 dias; 3. breaking change custa >1 semana de porte.
  Sem gatilho disparado, fork é custo sem retorno ( rever em cada Wave ).
- **Estado dos gatilhos hoje:** nenhum disparado. A11y quebrada no Freya é débito
  registrado (fora de V1), não bloqueador.
