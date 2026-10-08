# Matriz de aproveitamento — VectorCraft, PhotoCraft e LightCraft

**Status:** recomendações de integração, **não plano de merge de código**, não novas dependências aprovadas e não funcionalidades implementadas. Referência estática fixada em 2026-10-08 nos SHAs dos estudos individuais. Priorização qualitativa: ALTA/MÉDIA/BAIXA = **valor potencial relativo**, não pontuação de maturidade upstream ou promessa de tempo.

## 1. Premissa

Este estudo evita "portar os três aplicativos" ou criar um motor que dispute o Petunia. Um componente só se integra se respeitar `Petunia Core → Engine → RenderModel/Render → Qt UI`, invariantes de PTND, IDs autorais estáveis, Color Management e teste headless/visual.

**Classes de decisão:**
- **Estudar algoritmo:** seguir comportamento/estratégia, implementar API própria, sem copiar source.
- **Adaptar código:** portar partes localizadas sob licença compatível, attribution e isolamento.
- **Avaliar dependência:** usar crate upstream por commit/version fixo, cargo audit e adapter, somente se benchmark justificar.
- **Rejeitar arquitetura:** aprender com trade-off, mas não adotar tipo persistente, frontend, formato ou invariantes divergentes.

Nenhuma linha abaixo implica decisão automática de copiar código. A política para approvals de dependências/ADR segue [Política de dependências](#/docs/00-philosophy/dependency-policy.md).

## 2. Mapeamento por capacidade e ownership

| Função Petunia | Referência com path upstream | Tipo de aproveitamento | Destino Petunia | Prioridade | Incompatibilidades / provas obrigatórias |
|---|---|---|---|---|---|
| Cubic Bézier/anchors | VC `geom/src/path.rs` | Estudar, adapter | Engine Geometry | ALTA | `Corner/Smooth` vs `Cusp/Smooth/Symmetric`, IDs estáveis |
| Boolean/Pathfinder | VC `pathops/src/boolean.rs` | Comparar engine | Geometry/Pathops | ALTA | `linesweeper` vs `i_overlay`; open path, winding, precision |
| Planar subdivision | VC `pathops/src/planar.rs` | Estudar/adaptar | RegionGraph | ALTA | open boundaries, provenance, exact faces, degenerate edges |
| Shape Builder/Live Paint | VC `tools/src/builder.rs` | Fluxos/algoritmo | Smart Region | ALTA | LivePaint armazenado como Group nomeado vs RegionPaintObject tipado |
| Pen/Node gestures | VC `tools/src/pen.rs`, `direct.rs` | Estudar | UI ToolControllers | ALTA | Bend deve ser explícito, Alt não obrigatório, multi-node |
| Simplify/curve fitting | VC `pathops/src/edit.rs`, `fit.rs` | Algoritmo/adaptação | Smart Path Engine | ALTA | topology preservation, max error e conservação de NodeId |
| Width/Profile | VC `effects/src/stroke/width.rs` | Estudar/adaptar | Stroke Engine | MÉDIA-ALTA | arc length, joins/cusps e authoring |
| Live Blend/Repeat | VC `doc/src/blend.rs`, `effects/src/live.rs` | Estudar | Distribution Engine | MÉDIA-ALTA | instances virtuais, deterministic expansion, source bindings |
| Perspective/Warp/Mesh | VC `tools/src/distort/*`, `meshedit.rs` | Estudar | Transform/Render | MÉDIA | pós-milestone, CMM e geometric validity |
| SVG/PDF/CAD | VC `svg`, `pdf`, `eps`, `cad`, `metafile` | Avaliar adapters | Import/Export | MÉDIA | fidelity corpus, license, roundtrip |
| Sparse COW Tiles | PC `raster/src/lib.rs` | Adaptar conceito | Core Raster / Engine / Render | **MUITO ALTA** | format, memory, 256², mask defaults, dirty regions |
| Brush Dynamics | PC `paint/src/brush.rs`, `dynamics.rs` | Estudar/adaptar | Brush Engine | ALTA | pointer normalization, presets, pressure, flow vs opacity |
| Raster compositor | PC `compose/src/lib.rs`, `gpu/src/lib.rs` | Estudar | Render Model/Render | **MUITO ALTA** | display RGB upstream vs linear Petunia, CPU/GPU parity |
| Layer Effects/Adjustments | PC `doc/src/effects.rs`, `compose/src/adjust.rs` | Estudar/adaptar | Appearance/Render | ALTA | masks, clipping, styles, CMYK/ICC |
| PSD/PSB I/O | PC `psd/src/*`, `format/src/lib.rs` | Avaliar adapter | Import/Export | ALTA | untrusted input, unknown blocks, fidelity/roundtrip |
| Raster selection/retouch | PC `engine/src/selection_cmds.rs`, `paint/src/retouch.rs` | Estudar | Paint Persona | ALTA | tiles COW, selection masks, cancel/undo |
| Path edit in raster | PC `vector/src/edit.rs` | Estudar UX | Vector Edit | BAIXA-MÉDIA | usa índices `[subpath,knot]`; Petunia já possui modelo melhor |
| Render non-destructive photo | LC `develop/src/settings.rs`, `pipeline/src/lib.rs` | Adaptar algoritmo | Photo Adjustments | ALTA | stages, profile conversion, cached rendering |
| Mask evaluation | LC `pipeline/src/masks.rs` | Adaptar políticas | Mask Engine/Photo Persona | ALTA | Add=max; operators diferem por contexto |
| Preview cache & jobs | LC `preview/src/{lib,pool,disk,lru}.rs` | Adaptar conceito | IO Jobs / Render Cache | **MUITO ALTA** | hashes, invalidation, colorspace, memory/disk budget |
| RAW codecs & metadata | LC `raw/src/*`, `meta/src/*` | Avaliar dependência | Import/Resource | MÉDIA-ALTA | camera-specific corpus, security, binaries |
| Journaling/recovery | LC `catalog/src/journal.rs` | Estudar | PTND Persistence | ALTA | catalogue log ≠ document; atomicity across tiles/blobs |
| GPU staged develop | LC `gpu/src/lib.rs`, `docs/gpu-pipeline.md` | Estudar | Render Pipeline | MÉDIA-ALTA | hybrid CPU fallback, device selection, color |
| Subject/sky segmentation | LC `segment/src/lib.rs` | Experimento condicionado | Optional AI Masks | MÉDIA (futuro) | Apache-only code, Meta SAM weights non-OSI, hardware/opt-in |
| Unified Command/MCP | VC `engine/mcp`, PC `engine/automation`, LC `engine/mcp` | Integrar princípios | Engine + Plugin/MCP boundary | **MUITO ALTA** | typed DTOs, auth, capabilities, plugin sandbox |
| Plugin WASM runtime | VC/PC `plugins/src/lib.rs` | Avaliar runtime | Plugin Engine | ALTA | wasmi budgets, capability model, ABI versioning |
| GUI / keyboard | todos `ui-egui/src` | Inspiração funcional | Qt/QML | MÉDIA | não portar widgets egui, usar design system Petunia |

Abreviações: VC=VectorCraft, PC=PhotoCraft, LC=LightCraft. Cada path vem de árvore e/ou arquivos examinados; paths de submódulos não lidos integralmente são **pistas de estudo**, não garantia de funcionamento.

## 3. Workspaces/Personas Petunia

### Vector

**Módulos recomendados:** VC geom, pathops, builder, direct, pen, stroke width, live effects, trace e formats. Base de Smart Path, Smart Region, Transform, Appearance, Shape Builder e Pen. Para QML usar overlays derivadas de ToolResponse, não port de egui. As operações `Node`, `Bend`, `Region Paint` e `Select` devem manter ActionIds/contextos de UI definidos nas páginas canônicas.

### Paint / Raster

**Módulos:** PC raster, paint, compose, GPU, selection/adjusts. Abrange tiles COW, brushes, masks, blend modes, retouch, adjustment layers. LC masks pode complementar workflows de ajustes locais em fotografia, desde que seus operadores sejam normalizados às convenções do Paint/Mask Petunia.

### Photo / Develop (futuro conforme roadmap)

**Módulos:** LC develop, pipeline, GPU, RAW, meta, preview, segment optional. A Photo Persona usa um ImageResource source com effect stack não destrutiva. Library/catalog extenso somente se funcionalidade DAM for aprovada — não acoplar o editor inteiro a banco/catálogo.

### Layout / Document

**Módulos:** VC PDF/Text/Export e PC PSD/Smart Objects/Color; partes de documento multiartboard e gerenciamento de sources. O layout continua usando o SceneGraph Petunia, Styles/Symbols, Text Layout e PTND; não fundir document trees de terceiros.

### Automação / Plugins

**Módulos:** os três registries de Engine, VC/LC MCP Remote/Headless, PC automation/auth roots e VC/PC wasmi. **Um único dispatcher Petunia** com comandos tipados, overlays testáveis e export headless, UI autoral Qt via CXX-Qt.

## 4. Contratos do adapter de código

Exemplo conceitual (não é assinatura existente nem implementação real):

~~~rust
pub trait VectorBooleanBackend {
    fn apply(&self, req: BooleanRequest<'_>) -> Result<BooleanResult, GeometryError>;
}

pub trait RasterTileStore {
    fn read_snapshot(&self, id: PixelLayerId) -> Result<TileSnapshot, TileError>;
    fn apply_patch(&self, tx: RasterPatch) -> Result<DirtyTileSet, TileError>;
}

pub trait PhotoDevelopBackend {
    fn evaluate(&self, req: DevelopRequest<'_>, cancel: &CancelToken)
      -> Result<DevelopedBuffer, DevelopError>;
}
~~~

Na fronteira:
1. validar input finito, types, scopes e limits;
2. converter modelo Petunia → representação de algoritmo;
3. executar sem UI/Qt, budget/cancel;
4. converter saída com validadores `NodeId`/`ContourId`/FillRule/color space/provenance;
5. mostrar preview sem mutar documento;
6. Commit via uma Transaction com revision guard;
7. benchmark/testar mesmas fixtures em backends diferentes.

Não persistir struct external crate, pointers, `egui::Id`, `kurbo::BezPath` ou `photocraft::Surface` como verdade do documento. Não importar outros `CommandSpec` como catálogo paralelo.

## 5. Etapas de integração recomendadas

| Etapa | POCs e evidência | Gate de conclusão |
|---|---|---|
| **P0 — Auditoria técnica** | estudo estático destes três refs, upstream frozen commit e mapa por função | evidência/limites explicitados, sem alterações no código |
| **P1 — Adaptadores puros** | VC VectorPath↔BezPath; PC TileSnapshot; LC DevelopSettings mapping | invariantes/ids/enum/color/roundtrip |
| **P2 — Núcleo algorítmico** | boolean/region/refit; brush/tile/CPU compose; preview/stage cache | unit + property + fuzzy + paridade numérica |
| **P3 — Commands/History/IO** | Preview/Commit, Snap/Tool actions, PSD/RAW/Document APIs | cancel, stale revision, atomic Undo, migrations |
| **P4 — UI e acessibilidade** | overlays/Qt controllers, context bars, inspector schemas | keyboard, pointer/stylus, screen reader, High Contrast |
| **P5 — Bench/fidelidade** | real design docs, 10k+ nodes, 256² tiles, 4k/8k images, stress files | p95/memory budgets documentados e regressão contínua |

Essas etapas são **sequência técnica recomendada**, não compromisso de release nem autorização implícita para novas dependências.

## 6. Regras de decisão por problema

**Se há algoritmo equivalente aprovado (ex. `i_overlay`):** primeiro compare antes de substituir ou adicionar outra crate. Benchmark usa mesmo dataset/FillRule e critérios de precisão; não escolher pelo número de stars.

**Se há formato compatível de terceira parte (PSD/AI/RAW):** adapter de import/export separado, sandbox/limits e regressão de corpus, nunca trocar PTND.

**Se há plugin runtime de terceiro:** per-feature capability, no arbitrary fs/network by default, fuel/memory/deadline, fresh execution context e API versionada; budget e fallback.

**Se há mais de um modelo de cor:** decidir ponto exato de conversion, premultiplication e gamut mapping, protegendo scene-linear e ICC do Petunia.

**Se a ferramenta depende de UI:** portar somente design/gesto como `ToolController`+ `ActionId` e cobrir A11y, sem embutir egui.

## 7. Metadados da avaliação por item

Para cada candidato abrir um pequeno relatório técnico com:
- `source_project`, `source_commit`, `source_path`, `license`, `copyright_notice`;
- `confirmed behavior` com evidência concreta de código;
- `proposed target module`, `interface contract`, `alternatives`;
- `color/coord/identity semantics`, `data provenance`, `compatibility break`;
- `tests` (unit/property/fuzz/visual/bench), `known risks`;
- `decision`: reject / inspiration / experiment / approved dependency / adopted;
- commit/ADR/link de implementação Petunia quando existir.

Não marcar `adopted` ou `implemented` antes de merge + testes. Se um source mudar, comparar upstream SHA antes de revisar conclusões.

## 8. Referências canônicas

[VectorCraft](#/docs/06-references/vectorcraft.md) · [PhotoCraft](#/docs/06-references/photocraft.md) · [LightCraft](#/docs/06-references/lightcraft.md) · [Agent protocol](#/docs/06-references/agent-research-protocol.md) · [Arquitetura](#/docs/00-architecture/boundaries.md) · [Verificação](#/docs/00-architecture/verification.md).
