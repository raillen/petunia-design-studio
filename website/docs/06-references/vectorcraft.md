# VectorCraft — auditoria técnica aplicada ao Petunia

**Snapshot analisado:** [storytold/vectorcraft@5f92f5eb](https://github.com/storytold/vectorcraft/tree/5f92f5eb7fd194826aab2ad0844e474f13990938), 2026-10-08. **Status da evidência:** arquivos Rust, manifest, documentação técnica, arquivo de testes e árvore inspecionados; execução/benchmark não realizados. Não confundir claims de README com comportamento validado no Petunia.

## 1. Perfil

- Workspace Rust 2024, versão 0.7.0, 21 crates em `crates/`, três apps (desktop, CLI, web), `egui 0.36`.
- `kurbo = 0.13`, `peniko = 0.6`, `linesweeper = 0.4`, `vello_cpu = 0.2`, `usvg = 0.48`, `krilla = 0.8`, `wasmi = 2`, `skrifa`/`harfrust` e serde.
- Camadas observadas: `geom` e `doc` (dados), `pathops`, `brush`, `effects`, `text`, `trace` (algoritmos), `render`, `format/svg/pdf/eps/cad/metafile`, `tools`, `engine`, `mcp`, `plugins`, `ui-egui`, e apps shell.
- README se descreve como editor estilo Illustrator; anuncia Pathfinder, Shape Builder, Live Paint, Appearance, Blends, Gradient Mesh, Perspective, export e CLI/MCP. É **alegação funcional do upstream**; os módulos correspondentes existem em código, mas maturidade/qualidade não foi medida aqui.

## 2. Geometria: `crates/geom/src/path.rs`

[Arquivo original](https://github.com/storytold/vectorcraft/blob/5f92f5eb7fd194826aab2ad0844e474f13990938/crates/geom/src/path.rs).

`Anchor` tem coordenada absoluta `p`, `h_in`, `h_out` e `kind: AnchorKind`. O enum observado oferece **Corner** e **Smooth**, e representa handle ausente como controle coincidente com anchor (wire form omite valores iguais). `PathData` agrega `SubPath`, com `FillRule::NonZero/EvenOdd` e conversão para segmentos kurbo.

**Divergência crítica:** Petunia usa `NodeKind::Cusp/Smooth/Symmetric`, `Option<Point>` para handles, `NodeId/ContourId` estáveis e Line+Cubic explícito. Não importar `Anchor` no Core e não degradar Symmetric a Smooth sem preservação da intenção. Um adapter para `kurbo::BezPath` / algoritmos aceita resultado derivado mas mantém ids, kinds, contours, orientation e histórico autoral por provenance.

**Verificações propostas:** roundtrip adapter em contours abertos/fechados, handles ausentes/zero, Symmetric, self-intersection, NonZero, parents com transform não uniforme, split/reverse que preserva IDs e erro numérico declarado.

## 3. Boolean e Pathfinder: `crates/pathops`

Arquivos verificados:
- [`boolean.rs`](https://github.com/storytold/vectorcraft/blob/5f92f5eb7fd194826aab2ad0844e474f13990938/crates/pathops/src/boolean.rs) — `BoolOp::Union/Intersect/Difference/Xor`, arrangement por `linesweeper::topology`, split nos cruzamentos/extremos e fusão/refit de fragmentos de curva.
- [`planar.rs`](https://github.com/storytold/vectorcraft/blob/5f92f5eb7fd194826aab2ad0844e474f13990938/crates/pathops/src/planar.rs) — construção de faces/arestas por sweep, suporte a caminhos abertos como boundaries, tratamento de arestas coincidentes, nós half-edge e `live_paint`/`shape_builder`.
- [`edit.rs`](https://github.com/storytold/vectorcraft/blob/5f92f5eb7fd194826aab2ad0844e474f13990938/crates/pathops/src/edit.rs) — `SimplifyOptions` (tolerância em pontos, proteção de corners por ângulo, opção de linhas retas), fitting de cubics e funções de edição.
- `pathfinder.rs`, `offset.rs`, `fit.rs`: investigar para regras de offset, joins, tolerância, tratamento de curvas e regressão.

**Ponto forte:** `boolean.rs` opera sobre cubics e tenta reduzir "node explosion" após splits. Isso é diretamente útil aos problemas do Petunia em contornos com pontos demais, Shape Builder e Clean Vector. `planar.rs` trata **open paths como separadores de faces**, característica necessária a Region Paint.

**Cuidado 1:** cabeçalho de `boolean.rs` informa que subpaths abertos são implicitamente fechados quando interpretados como preenchimento. Petunia só pode reproduzir isso quando Fill policy e o contexto exigirem; **não fechar a geometria autoral** de um contour aberto como efeito lateral.

**Cuidado 2:** `linesweeper` versus `i_overlay` já aprovado no Petunia. Antes de adicionar segundo boolean engine, criar **spike fora do Core** para comparar robustez, licença/transitive deps, desempenho, mapeamento de FillRule e curvas preservadas. Não substituir `i_overlay` por intuição. Avaliar qual pode fornecer faces/open boundaries com menor custo de adaptação.

**Cuidado 3:** a tolerância de fitting do upstream não vira default universal no Petunia: unidades, zoom, escala, perfil de export e erro máximo visual mudam a adequação. Separar tolerâncias geométricas autorais das tolerâncias de hit-test em pixels lógicos.

## 4. Smart Region: `crates/tools/src/builder.rs`

[Arquivo](https://github.com/storytold/vectorcraft/blob/5f92f5eb7fd194826aab2ad0844e474f13990938/crates/tools/src/builder.rs).

O módulo implementa controllers para Shape Builder, Live Paint Bucket e Live Paint Selection e emite comandos `shapeBuilder.merge`, `livePaint.fill`, `livePaint.strokeEdge` e `select.*`. Um `BuilderMap` e caching de regiões conectam hit-test de faces à geometria de `pathops::shape_builder`/`live_paint`.

**Não copiar seu modelo de Live Paint:** o código descreve grupos comuns identificados por prefixo no nome, subgrupo oculto das sources e child paths para faces/edges. Petunia já definiu `RegionPaintObject` e bindings/provenance tipados; uma string no nome de um Group não pode ser o discriminante semântico de um objeto. Adotar os algoritmos/gestos de referência, não o modo de persistir o recurso.

**Adaptação desejada:**
`Sources → normalized intersections → derived RegionGraph{faces,edges,adjacency,provenance} → RegionRef(hit) → LiveBuild / RegionPaint bindings / CloseGap query → Transaction / RenderSnapshot`.

Analisar edges abertos, degenerate co-linears, face ambiguities, clip/masks, merges com diferenças de fill style, regeneração após edição source e invalidation por revision.

## 5. Pen e Direct Selection: `crates/tools/src/pen.rs` e `direct.rs`

- [`pen.rs`](https://github.com/storytold/vectorcraft/blob/5f92f5eb7fd194826aab2ad0844e474f13990938/crates/tools/src/pen.rs): stateful Pen, criação/continuação/fechamento, rubber-band, snap, drag de tangentes, alternância temporária de ferramenta e Auto Add/Delete.
- [`direct.rs`](https://github.com/storytold/vectorcraft/blob/5f92f5eb7fd194826aab2ad0844e474f13990938/crates/tools/src/direct.rs): seleção de anchors/handles/segmentos, marquee, snap/smart guides, corners, spine de blends, meshes e transformações.

**Diferença de UX intencional:** `direct.rs` descreve deformação por drag de segmento quando curved; no Petunia, **Node não deve deformar segmento pelo arrasto sem ativar Bend**. O arquivo de Pen descreve Alt para quebrar handle; no Petunia, `Unlink Tangents` deve ser ActionId acessível, sem depender de Alt reservado pelo WM Linux. Auto Add/Delete nunca deve surpreender quem está criando um path. Preservar decisões canônicas em [Vector Edit](#/docs/04-ui/vector-edit-interaction.md), [Precision](#/docs/04-ui/vector-edit-precision.md) e [Pen](#/docs/04-ui/pen-path-creation.md).

**Valor técnico:** reconhecer estados e overlays, respeitar pointer capture, converter gestos em comandos, testar snap durante draw/drag e casos de degenerados. O Petunia deve usar seus próprios `ToolController`, `SelectionState`, `TransientEdit` e tipos de hit test.

## 6. Simplify, Smooth, Clean e operações assistidas

[`pathops/edit.rs`](https://github.com/storytold/vectorcraft/blob/5f92f5eb7fd194826aab2ad0844e474f13990938/crates/pathops/src/edit.rs) contém `simplify_with` e fitting de segmentos, preservação de corners por limite angular e proteção para não retornar subpath com mais anchors que a origem. [`engine/cmd/pathops.rs`](https://github.com/storytold/vectorcraft/blob/5f92f5eb7fd194826aab2ad0844e474f13990938/crates/engine/src/cmd/pathops.rs) registra Pathfinder, Offset Path, Outline Stroke, Simplify, Add Anchors, Clean Up etc. Separar **redução de anchors** de **filtragem de defeitos topológicos** no Petunia.

**Reuso recomendado:** extrair padrões de solver e testes de fitting para Smart Delete/Simplify/Direct Bend, preservando ids de nodes sobreviventes e critérios de erro. Para casos de contours autointersectados, boolean com mudança de winding e pequenas ilhas, anexar golden fixtures e provas de paridade preview/commit.

## 7. Appearance, paint, mesh, effects, render

Pistas verificadas na árvore:
- `crates/doc/src/appearance.rs`, `blend.rs`, `pattern.rs`, `perspective.rs`, `recolor.rs`, `puppet.rs`.
- `crates/effects/src/stroke/width.rs`, `warp.rs`, `distort.rs`, `stroke/outline.rs`.
- `crates/tools/src/meshblend.rs`, `meshedit.rs`, `distort/width.rs`, `distort/perspective.rs`.
- `crates/color/src/recolor.rs`, `harmony.rs`, `cms/icc.rs`.
- [`crates/render/src/lib.rs`](https://github.com/storytold/vectorcraft/blob/5f92f5eb7fd194826aab2ad0844e474f13990938/crates/render/src/lib.rs): CPU `vello_cpu` com premultiplied RGBA, culling por painted bounds, fills/strokes, blend modes, clip groups, gradientes, imagem e texto. `render_region` tem limites explícitos de raster.

**Para Petunia:** investigar as soluções geométricas de Variable Width, Gradient Mesh, Live Blend, radial repeat, Perspective/Envelope e aparência. Não importar render architecture tal como está sem avaliar `RenderSnapshot`, gerenciamento de cor definido e separação Engine↔Render. Um backend `vello_cpu` pode ser opção via adapter, **não aprovação imediata nem necessidade de substituir o render planejado**.

## 8. Comandos, History e ferramentas

[`crates/engine/src/lib.rs`](https://github.com/storytold/vectorcraft/blob/5f92f5eb7fd194826aab2ad0844e474f13990938/crates/engine/src/lib.rs) documenta `Session::execute` como entrada única para IDs estáveis de comandos + params JSON. UI, CLI, canal remoto e MCP compartilham dispatcher; ferramentas traduzem pointer gestures em comandos. O source contém `History`, `HistoryEntry`, `UndoGroup`, `Interaction` e `DocState`.

**Adotar conceito, não payload:** Petunia já tem ActionIds e DTOs tipados, Transaction e History. Registrar no Catálogo de ações um binding tipo `CommandInfo` (nome, contexto, enabled state, schema, docs, error reasons, Undo policy). JSON no limite MCP/CLI; dentro do Engine preferir tipos Rust estáveis e validação central.

## 9. MCP e plugins

[`crates/mcp/src/lib.rs`](https://github.com/storytold/vectorcraft/blob/5f92f5eb7fd194826aab2ad0844e474f13990938/crates/mcp/src/lib.rs) observa duas implementações do mesmo protocolo: `Remote` para desktop loopback e `Headless` para Session in-process, ambas por MCP stdio JSON-RPC. Exemplos testados de comando estão nos modules `crates/mcp/src/tests_*.rs`.

[`crates/plugins/src/lib.rs`](https://github.com/storytold/vectorcraft/blob/5f92f5eb7fd194826aab2ad0844e474f13990938/crates/plugins/src/lib.rs) e [`docs/plugins.md`](https://github.com/storytold/vectorcraft/blob/5f92f5eb7fd194826aab2ad0844e474f13990938/docs/plugins.md): plugin `wasmi` ABI v1, sem host imports, sem WASI/network/fs, com fuel, memory/recursion limits, deadline e fresh instance. Dois tipos: object filters (edits atômicos) e live effects (reavaliação por appearance).

**Compatibilidade:** aproveitar threat model e testes, não importar ABI como padrão do Petunia sem avaliação. O Petunia precisará Host API versionada, capability-based permission, context isolation, budget, cancellation e adapters tipados, compatíveis com seu PTND.

## 10. Testes, maturidade e riscos

Diretórios observados: `crates/pathops/src/*`, `crates/tools/src/*`, `crates/render/src/tests*.rs`, `crates/mcp/src/tests*.rs`, CLI integration tests `apps/vectorcraft-cli/tests/`. `AGENTS.md` institui clean-room, assets attribution e zero-panics em código distribuído; `Cargo.toml` nega unsafe e muitos macros de panic no Clippy. Bom modelo de disciplina, não prova de ausência de bugs.

**Riscos antes de usar:**
1. Diff entre `kurbo` 0.13 e 0.11 e tipo Path do Petunia; encapsular adapter e/ou pesquisar update separately.
2. `linesweeper` e `i_overlay` podem duplicar responsabilidade; benchmark, robustez e manutenção precisam justificar alternativa.
3. Shape Builder com name-prefix groups é inadequado para persistência tipada e PTND.
4. Alterações automáticas de Pen/Direct divergem da política de gestos previsíveis e acessíveis aprovada.
5. Claims de desempenho (`20k shapes ~27ms` no README) dependem de hardware/cena; não entram como baseline Petunia.
6. Export PDF/AI/EPS/EMF/WMF exige checagem de licenças, standards, roundtrip e fidelidade, não apenas existência de módulos.
7. Ícones e screenshots upstream têm licenças distintas do código. Não importar branding/ativos em UI.

## 11. Experimentos de integração (em ordem)

**VC-01 — Geometry adapter.** Roundtrip `VectorPath↔BezPath` com invariantes Petunia; Symmetric/IDs/handles/FillRules intactos.
**VC-02 — Boolean benchmark.** Casos sintéticos e arte reais (holes, overlaps, almost-coincident, curves), comparar `i_overlay`/abordagem de `linesweeper`, tempo/erro/nodes.
**VC-03 — RegionGraph.** Prototype faces de open paths, provenance, remap após source edit e gap virtual, sem portar group-by-name.
**VC-04 — Simplify/refit.** Medir Hausdorff/chord deviation e node reduction com topology preservation.
**VC-05 — Tool gesture audit.** Testar Pen/Direct/Smart Guides como *referência de fluxo*, implementar Actions Petunia com accessibility.
**VC-06 — Headless/Plugin.** API de comando e teste de sandbox em harness do Petunia; sem interface egui.

**Aceitação comum:** 1 comando = transação recuperável, preview/commit parity, IDs estáveis, cancel/stale revision, no panic, a11y sem ponteiro e funcionamento headless. [Matriz detalhada](#/docs/06-references/integration-matrix.md).

## 12. Referências do próprio Petunia

[Core Path](#/docs/01-core/path.md) · [Smart Region](#/docs/04-ui/smart-region-interaction.md) · [Smart Path](#/docs/04-ui/smart-path-operations.md) · [Vector precision](#/docs/04-ui/vector-edit-precision.md) · [Render model](#/docs/03-render/render-model.md).
