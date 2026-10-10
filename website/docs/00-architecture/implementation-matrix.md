# Matriz de implementação

A matriz registra **arquivos existentes**, responsabilidades atuais e módulos necessários para chegar a um editor profissional sem criar god modules.

## Estado atual e alvo fechado

A documentação de arquitetura/Core/Engine/Render já define o alvo. A tabela abaixo separa **implementação atual** de **contrato técnico fechado**, evitando tratar uma decisão já tomada como “próxima decisão”.

| Área | Implementação atual (2026-10-10) | Contrato ainda pendente |
|---|---|---|
| Core / Color | RGB/CMYK/Gray/Lab, espaços explícitos, Spot, Swatches e gradients | policies profissionais de saída e proofing |
| Core / Math + Path | f64, transforms, Line/Cubic, handles unilaterais, IDs e validação | guards de flatten em todos os consumidores legados |
| Core / Generated Content | modelos persistentes; Engine avalia Trace/QR/Barcode | comandos de Expand e ferramentas de parâmetros |
| Core / Styles + Symbols | registries, text styles, símbolos e overrides tipados | ligação de AppearanceSource aos objetos, resolução de StyleId/overrides e detach |
| Core / Guides + Grids + Slices | modelos e invariantes; affine/baseline/perspective grids | autoria completa, providers locais e todos os tipos de snap |
| Core / Crop + Scene + Document | crop, bindings, hierarquia, Pages/Spreads/Artboards e validação | trim destrutivo e handlers de autoria completos |
| Engine / Commands | transactions staged, revisões inéditas, undo/redo, pruning e hard budget | handlers dos controles ainda não implementados |
| Engine / Geometry | Bézier, booleans, offset, simplify, curve fit e Shape Builder | geometry effects integrados e guards em APIs legadas |
| Engine / Spatial | R*-tree, world/page/visibilidade, hit de snapshot e hysteresis | perspective/baseline snap, guides em artboards e markers |
| Engine / Brush/Raster | dabs reais, tiles COW, preview/cancel, PNG transacional e filtros ROI | tools raster e GC de recursos conforme retenção do histórico |
| Engine / Text/Layout | BiDi, shaping, fallback, estilos, wrap, hyphenation, outlines, on-path e linked frames | baseline grids, layout editorial e caret/seleção tipográfica |
| Engine / Color | Little CMS 2 estático encapsulado, ICC RGB/CMYK/Gray/Lab e cache | perfis de entrada/saída de imagens, proofing e gamut |
| Engine / I/O/Plugins | PNG/JPEG limitados, SVG path serializer, negotiation, scheduler, WASM real e MCP | importador SVG, exportadores de documento e Host API completa |
| Engine / Persistence | PTND ZIP/ZIP64, save snapshot, lock OS, conflito, previews/extensions e recovery | lifecycle de autosave/recovery e guards de sessão/recurso para jobs |
| Engine / Fragments | closure, recursos/registries/grids, remapping e paste/undo atômico | policies de reuso externo e novas entidades conforme handlers surgirem |
| Render Model | crate presente; snapshots imutáveis e recursos derivados | ampliar contrato para providers adicionais |
| Render | software tiled, fill rules, paints, strokes, clips/masks, blur/shadow e pixels de recursos | effect-instance compositor, geometry effects, vector patterns, markers e pool/cache no pipeline |
| UI | sessão headless e adaptador petunia-desktop Qt/QML/CXX-Qt; canvas, painéis, personas, preferências e arquivos | QA com tecnologias assistivas, GPU e plataformas; pintura/texto sem controlador de janela |

As regressões em `audit_regressions.rs` fecham os bugs reproduzidos na auditoria inicial. **Não fecham os contratos pendentes desta tabela.** Recursos e efeitos sem avaliação fiel produzem diagnóstico explícito e preservam os dados autorais.

> **Atualização 2026-10-09:** a discussão conjunta foi realizada e o contrato de interação está aprovado no [ADR-0012](#/docs/00-architecture/adr/0012-interaction-contract.md) (hit-test, context bar, targets, atalhos, tooltip, aparência, Smart Delete, campos numéricos, workspace, ponte Qt). A UI continua fora do fechamento **técnico** de Core/Engine/Render; o que mudou é que a camada de interação agora tem direção única. A janela Qt/QML foi implementada em `petunia-desktop` em 2026-10-10; [execução e limites](#/docs/04-ui/native-desktop.md). QA assistivo e multiplataforma permanece aberto.

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

## Refatoração ainda necessária

A extração de `app/runtime.rs` separou os adapters de persistência/providers/clipboard. Ainda há concentração de responsabilidades em `compile.rs`, `fragments.rs`, `software.rs` e `app.rs`. Antes de ampliar os handlers e a ponte GUI, extrair fases de compilation/effects/resources, separar closure/remapping de fragments, separar sampling/composição de pixels e reduzir os adapters da sessão. A refatoração deve preservar as regressões de saída; tamanho de arquivo sozinho não comprova uma responsabilidade incorreta.

Os caches/pools precisam de integração real com invalidação e limites; capacidades reservadas no construtor não provam uso. Consumers legados de flatten devem migrar para APIs checked, e resultados de jobs devem validar identidade/revisão de sessão e recurso antes de publicar.

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
13. **Discussão de interação concluída pelo ADR-0012; executar e verificar Tools/Workspace/Acessibilidade/GUI conforme o contrato.**

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
| Tools/Workspace/Acessibilidade/GUI/UX | ✅ Contrato ADR-0012 aprovado; GUI funcional verificada em Linux/offscreen; QA assistiva/multiplataforma aberta |

“Definido” não significa “já implementado”. Significa que a implementação possui uma direção técnica única e critérios de correção suficientes para começar sem rediscutir a arquitetura a cada módulo.

## Decisão de interação Vector Edit

**✅ Modelo híbrido aprovado:** Select para objetos e hierarquia; Vector Edit contextual para nodes/segmentos/handles, com operação Node padrão e Bend/Cut/Width explícitas.

**✅ Fechado em 2026-10-09 pelo [ADR-0012](#/docs/00-architecture/adr/0012-interaction-contract.md):** precedência de hit-test, context bar, targets e âncoras, atalhos primários + editor reatribuível, tooltip, aparência/temas, Smart Delete, campos numéricos, workspace e ponte Qt.

**Entrega 2026-10-10:** tokens e QML concretos, sessão e adaptador Qt integrados e verificados conforme [GUI nativa](#/docs/04-ui/native-desktop.md). ContextStack/ToolControllers completos e QA de usabilidade nas plataformas finais continuam pendentes. A entrega funcional não fecha esses gates.

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

**Estado real dessas ferramentas criativas:** os contratos estão escritos; as famílias Smart/Advanced da tabela ainda precisam de implementação e verificação próprias. A GUI base já possui contrato aprovado e implementação funcional; isso não implementa automaticamente essas famílias.

## Decisões propositalmente não congeladas

Esses pontos **não foram esquecidos**. Permanecem abertos porque a filosofia do projeto exige evidência de implementação/profiling ou discussão de experiência:

| Tema | Motivo |
|---|---|
| Generational Arena | só entra se storage atual justificar por performance/ergonomia |
| Tile size raster/render | depende de cache locality, brush, blur e memória |
| Número de worker threads | depende de hardware e budget interativo |
| Threshold incremental vs rebuild do R*-tree | benchmark |
| Runtime WASM | wasmi escolhido e testado; medir startup/binário/sandbox/throughput |
| Backend GPU | pós software-reference; precisa preservar semântica |
| HDR completo | exige política de luminância/output real |
| Mesh Gradient avançado | precisa modelo próprio, não enum reservado |
| Tables/editorial avançado | pós-v0.1-stable |
| UI plugin extension | será discutida com GUI/UX |
| Tools, Workspace, Acessibilidade e GUI/UX | ~~discussão conjunta com o usuário~~ → **contrato aprovado (ADR-0012, 2026-10-09)**; GUI base implementada e verificada em 2026-10-10, QA de plataformas e tecnologias assistivas permanece aberto |

Uma decisão “aberta por evidência” não autoriza implementações incompatíveis. Os contratos ao redor já estão definidos.

