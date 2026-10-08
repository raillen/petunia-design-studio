# Matriz de implementação

A matriz registra **arquivos existentes**, responsabilidades atuais e módulos necessários para chegar a um editor profissional sem criar god modules.

## Estado atual e alvo fechado

A documentação de arquitetura/Core/Engine/Render já define o alvo. A tabela abaixo separa **implementação atual** de **contrato técnico fechado**, evitando tratar uma decisão já tomada como “próxima decisão”.

| Área | Implementação atual | Contrato técnico |
|---|---|---|
| Core / Color | RGBA + poucos espaços | valor + perfil/espaço explícito, Spot, Swatches, gradients |
| Core / Math | tipos básicos | f64 canônico, coordenadas Y-down, tolerâncias contextuais, transforms robustos |
| Core / Path | nodes cúbicos + contours | Line+Cubic, IDs estáveis, fill semantics; ParametricShape separado em `shape.rs` |
| Core / Generated Content | ausente | TraceObject + GeneratedVectorObject, source/params autorais e geometria derivada |
| Core / Styles + Symbols | parcial | styles flat/linkable com overrides tipados; symbols acíclicos, nested e detach explícito |
| Core / Guides + Grids + Slices | parcial | guides tipadas; affine/baseline/perspective grids; slices reproduzíveis sem path absoluto |
| Core / Crop + Clipping | parcial | Image source_rect normalizado + ClipBinding; trim destrutivo separado |
| Core / Scene | Path/Group simples | hierarquia ordenada, bindings, symbols, generated content e atomic structural mutations |
| Core / Document | canvas + scene | Pages/Spreads/Artboards, registries, resources, styles e setup |
| Engine / Commands | undo/redo básico | Command → DocumentOp → Transaction → HistoryEntry |
| Engine / Geometry | parcial | Bézier, boolean, offset, simplify, smooth, cleanup, curve fit, Shape Builder e provenance |
| Engine / Spatial | snapping básico | R*-tree, hit-test, candidates, ranking, hysteresis e grids |
| Engine / Brush/Raster | parcial | One Euro, arc-length resampling, tiled COW, filters/ROI |
| Engine / Text/Layout | parcial | Unicode/BiDi/shaping/line layout/fallback/linked frames |
| Engine / Color | parcial | Little CMS 2 encapsulado, ICC transforms, proofing e cache |
| Engine / I/O/Plugins | parcial | import/export contracts, scheduler, WASM plugins, MCP adapter |
| Engine / Persistence | conceitual | save por snapshot, PTND ZIP/ZIP64, autosave separado, checkpoint + recovery journal |
| Engine / Fragments | ausente | DocumentFragment, dependency closure, ID remapping e copy/paste atômico |
| Render Model | ainda ausente | contrato imutável Engine → Render já definido arquiteturalmente |
| Render | software mínimo | tiled software reference renderer + render graph + compositor |
| UI | sessão mínima | **fora deste fechamento; volta para discussão conjunta** |

## Arquivos-alvo do Núcleo

```text
petunia-core/src/
├── color.rs
├── math.rs
├── units.rs
├── id.rs
├── path.rs
├── shape.rs
├── generated.rs
├── paint.rs
├── appearance.rs
├── effects.rs
├── scene.rs
├── text.rs
├── raster.rs
├── resources.rs
├── styles.rs
├── symbols.rs
├── guides.rs
├── document.rs
├── serialization.rs
└── error.rs
```

Não é necessário criar todos imediatamente. A regra é extrair quando a responsabilidade deixar de caber claramente no arquivo atual.

## Arquivos-alvo do Engine

```text
petunia-engine/src/
├── command/
├── geometry/
│   ├── bezier.rs
│   ├── bounds.rs
│   ├── intersections.rs
│   ├── boolean.rs
│   ├── offset.rs
│   ├── simplify.rs
│   └── shape_builder.rs
├── spatial/
│   ├── index.rs
│   ├── hit_test.rs
│   ├── snapping.rs
│   ├── alignment.rs
│   └── measurement.rs
├── brush/
├── raster/
├── text/
├── layout/
├── color/
├── import/
├── export/
├── persistence/
├── fragments/
├── jobs/
└── plugins/
```

## Arquivos-alvo do Render Model

```text
petunia-render-model/src/
├── snapshot.rs
├── primitive.rs
├── paint.rs
├── text.rs
├── image.rs
├── effect.rs
├── composite.rs
└── resource.rs
```

A crate contém apenas DTOs/runtime contracts imutáveis de renderização. Sem backend, cache, Qt ou algoritmos geométricos.

## Arquivos-alvo do Render

```text
petunia-render/src/
├── backend.rs
├── frame.rs
├── graph.rs
├── vector/
├── raster/
├── text/
├── compositor/
├── effects/
├── cache/
├── overlay/
├── output/
└── software.rs
```

## Arquivos-alvo da Interface

```text
petunia-ui/src/
├── app.rs
├── session.rs
├── input.rs
├── actions.rs
├── tools/
├── viewport/
├── panels/
├── workspace/
└── accessibility/
```

Qt/QML e CXX-Qt permanecem na borda de UI. Nenhum tipo Qt deve entrar em Core, Engine ou Render.

## Ordem de implementação recomendada

1. Consolidar Core: math/units/ids/path/shape/color/appearance/scene/document/resources.
2. Implementar Transactions/History sobre o Core fechado.
3. Implementar Geometry + Spatial e seus adapters.
4. Introduzir `petunia-render-model` e compiler/evaluator Engine → RenderSnapshot.
5. Implementar software tiled renderer, compositor e output.
6. Implementar raster tiles/brush/filter pipeline.
7. Integrar Color Management com Little CMS 2.
8. Completar Text/Layout + font resolution/outlines.
9. Consolidar PTND ZIP/ZIP64, save/load/migrations, recovery e DocumentFragment/copy-paste.
10. Consolidar import/export e capability negotiation.
11. Implementar scheduler/jobs, plugin Host API WASM e MCP adapter.
12. Executar diagnostics/observability, verification, benchmarks e fuzzing; corrigir gargalos.
13. **Parar antes de Tools/Workspace/Acessibilidade/GUI e retomar discussão conjunta.**

A ordem privilegia invariantes e contratos antes da camada de interação.

## Fechamento técnico antes da UI

A especificação de arquitetura/motor é considerada **fechada o suficiente para implementação incremental** quando todas as áreas abaixo possuírem contrato, invariantes e quality gate:

| Área | Status documental |
|---|---|
| Fronteiras, estado, concorrência e scheduler | ✅ Definido |
| Precisão, determinismo e matemática base | ✅ Definido |
| IDs, serialização e PTND | ✅ Definido |
| Não destrutibilidade/evaluation | ✅ Definido |
| Core Path/Shape/Color/Appearance | ✅ Definido |
| Core Scene/Document/Text/Raster/Resources | ✅ Definido |
| Core Styles/Symbols | ✅ Definido |
| Guides/Grids/Slices + Crop/Clipping | ✅ Definido |
| Conteúdo gerado: Trace/QR/Barcode | ✅ Definido |
| Commands/History + memory budget | ✅ Definido |
| Geometry + Shape Builder | ✅ Definido |
| Spatial/Hit-test/Snapping | ✅ Definido |
| Brush/Raster/Filters | ✅ Definido |
| Text/Layout/Unicode | ✅ Definido |
| Color Management | ✅ Definido |
| I/O/Export/Plugins/MCP | ✅ Contrato definido |
| Save/Autosave/Recovery | ✅ Definido |
| DocumentFragment/Copy-Paste | ✅ Definido |
| Render Model/Pipeline/Compositor/Paint/Adjustments | ✅ Definido |
| Cache/Output/Headless | ✅ Definido |
| Diagnostics/Verification/Security limits | ✅ Definido |
| Tools/Workspace/Acessibilidade/GUI/UX | ⏸ discussão conjunta |

“Definido” não significa “já implementado”. Significa que a implementação possui uma direção técnica única e critérios de correção suficientes para começar sem rediscutir a arquitetura a cada módulo.

## Decisão de interação Vector Edit

**✅ Modelo híbrido aprovado:** Select para objetos e hierarquia; Vector Edit contextual para nodes/segmentos/handles, com operação Node padrão e Bend/Cut/Width explícitas.

**⏸ Em discussão conjunta:** visual final, toolbar, atalhos secundários, UX de operações específicas e critérios finais de acessibilidade.

Detalhes: [Vector Edit — especificação](#/docs/04-ui/vector-edit-interaction.md) e [ADR-0011](#/docs/00-architecture/adr/0011-hybrid-vector-edit.md). O código atual ainda não implementa ContextStack e ToolControllers completos.

## Novo sistema de ferramentas criativas — contratos adicionados

As cinco famílias e as ferramentas avançadas foram aprovadas como escopo funcional documentado. A implementação ainda precisa seguir fundações, sem criar motores redundantes:

| Sistema | Core | Engine | Render | UI |
|---|---|---|---|---|
| Smart Path | Path/NodeId | fitting, simplify, cleanup, queries | overlays | proposta de ferramentas |
| Smart Region | Region bindings/provenance | planar arrangement, gap, weave | region mask/occlusion | proposta de ferramentas |
| Smart Distribution | DistributionObject e refs | live transforms/arc-length/blend | virtual instances | proposta de ferramentas |
| Smart Color | Swatches/ColorMappingSpec | extraction, color matching, constraints | live recolor | proposta de ferramentas |
| Smart Measure | DimensionObject/anchors | measurement + validation | annotations/overlays | proposta de ferramentas |
| Advanced | WidthProfile/Pattern/Effects/Warp specs | Stroke/Pattern/Brush/Warp | masks/effects | proposta de ferramentas |

A especificação canônica vive em [Core Models](#/docs/01-core/creative-features.md), [Creative Operations](#/docs/02-engine/creative-operations.md) e [Tools Overview](#/docs/04-ui/creative-tools-overview.md).

**Estado real:** contratos escritos no site; crates e funcionalidades ainda precisam ser codificadas e verificadas. A definição de GUI/UX ainda exige a discussão conjunta.

## Decisões propositalmente não congeladas

Esses pontos **não foram esquecidos**. Permanecem abertos porque a filosofia do projeto exige evidência de implementação/profiling ou discussão de experiência:

| Tema | Motivo |
|---|---|
| Generational Arena | só entra se storage atual justificar por performance/ergonomia |
| Tile size raster/render | depende de cache locality, brush, blur e memória |
| Número de worker threads | depende de hardware e budget interativo |
| Threshold incremental vs rebuild do R*-tree | benchmark |
| Runtime WASM concreto | medir startup/binário/sandbox/throughput |
| Backend GPU | pós software-reference; precisa preservar semântica |
| HDR completo | exige política de luminância/output real |
| Mesh Gradient avançado | precisa modelo próprio, não enum reservado |
| Tables/editorial avançado | pós-v0.1-stable |
| UI plugin extension | será discutida com GUI/UX |
| Tools, Workspace, Acessibilidade e GUI/UX | discussão conjunta com o usuário |

Uma decisão “aberta por evidência” não autoriza implementações incompatíveis. Os contratos ao redor já estão definidos.

