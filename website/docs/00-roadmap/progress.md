# Progresso das etapas

Página de acompanhamento das **etapas de implementação** do Petunia Design Studio. Ela distingue **contrato documental** (o que a arquitetura já definiu) de **estado no checkout** (o que o código realmente executa hoje).

> **Como ler:** `SPECIFIED` ≠ `IMPLEMENTED`. Nenhum percentual aqui é medição de cobertura. Os estados seguem a [taxonomia obrigatória](#/docs/07-agents/authority-reading.md) e a fonte da verdade é a [matriz de implementação](#/docs/00-architecture/implementation-matrix.md) + o checkout atual. Verificação manual em 2026-10-09 via `ls crates/*/src` e leitura de `lib.rs`.

## Resumo

<div class="progress-summary" role="region" aria-label="Resumo do progresso">
  <div class="progress-card">
    <strong>Contrato de arquitetura</strong>
    <p>Core, Engine, Render Model e Render <strong>fechados o suficiente</strong> para implementação incremental. UI/Tools/Workspace seguem <strong>pausados para discussão conjunta</strong>.</p>
    <span class="badge badge-specified">SPECIFIED</span>
  </div>
  <div class="progress-card">
    <strong>Estado no checkout</strong>
    <p>Fundação inicial presente nas 4 crates. Nenhuma etapa acima está `TESTED` ou `VERIFIED` por esta página — isso exige comando executado e evidência.</p>
    <span class="badge badge-partial">PARTIAL</span>
  </div>
  <div class="progress-card">
    <strong>Regra da página</strong>
    <p>Ao consolidar código, atualize a etapa correspondente e registre a evidência (caminho, commit, teste). Sem evidência, o estado volta para `UNVERIFIED`.</p>
    <span class="badge badge-unverified">EVIDÊNCIA OBRIGATÓRIA</span>
  </div>
</div>

## Etapas — ordem recomendada

Fonte da ordem: [matriz de implementação](#/docs/00-architecture/implementation-matrix.md) § "Ordem de implementação recomendada". Estado do checkout: verificação manual de 2026-10-09.

| # | Etapa | Contrato | Estado no checkout | Evidência |
|---|---|---|---|---|
| 1 | Consolidar Core: math/units/ids/path/shape/color/appearance/scene/document/resources | `SPECIFIED` | `PARTIAL` — 8 módulos presentes (`color, document, id, math, path, scene, units, error`); faltam `shape, generated, paint, appearance, effects, text, raster, resources, styles, symbols, guides, serialization` | `crates/petunia-core/src/lib.rs`, `units.rs` |
| 2 | Transactions/History sobre o Core | `SPECIFIED` | `PARTIAL` — undo/redo básico (`AddNodeCommand`, `TransformNodeCommand`, `CommandHistory`); sem `DocumentOp`, `Transaction`, `HistoryEntry` completos | `crates/petunia-engine/src/command.rs` |
| 3 | Geometry + Spatial e adapters | `SPECIFIED` | `PARTIAL` — snapping básico presente; sem Bézier/boolean/offset/simplify/R-tree/hit-test completos | `crates/petunia-engine/src/snapping.rs` |
| 4 | `petunia-render-model` + RenderSnapshot (Engine → Render) | `SPECIFIED` | `PLANNED` — crate ausente no workspace; contrato definido na arquitetura | [Render Model](#/docs/03-render/render-model.md) |
| 5 | Software tiled renderer, compositor e output | `SPECIFIED` | `PARTIAL` — contrato `RenderBackend` + referência mínima presentes; sem tiled graph/compositor/cache completos | `crates/petunia-render/src/backend.rs`, `software.rs` |
| 6 | Raster tiles / brush / filter pipeline | `SPECIFIED` | `PLANNED` — sem módulo brush/raster no checkout | [Matriz](#/docs/00-architecture/implementation-matrix.md) |
| 7 | Color Management (LittleCMS 2 encapsulado) | `SPECIFIED` | `PARTIAL` — `ColorRgba` + poucos espaços no Core; sem ICC/proofing/cache | `crates/petunia-core/src/color.rs` |
| 8 | Text/Layout + font resolution/outlines | `SPECIFIED` | `PLANNED` — sem módulo text/layout no checkout | [Matriz](#/docs/00-architecture/implementation-matrix.md) |
| 9 | PTND ZIP/ZIP64, save/load/migrations, recovery, Fragment/copy-paste | `SPECIFIED` | `PARTIAL` — `Document` + canvas/scene básicos; sem ZIP, journal, fragment | `crates/petunia-core/src/document.rs` |
| 10 | Import/export + capability negotiation | `SPECIFIED` (contrato) | `PLANNED` — sem módulos import/export no checkout | [Matriz](#/docs/00-architecture/implementation-matrix.md) |
| 11 | Scheduler/jobs, plugin Host API WASM, MCP adapter | `SPECIFIED` (contrato) | `PLANNED` — sem scheduler/plugins no checkout | [Matriz](#/docs/00-architecture/implementation-matrix.md) |
| 12 | Diagnostics/observability, verification, benchmarks, fuzzing | `SPECIFIED` | `UNVERIFIED` — sem evidência de execução registrada por esta página | [Verificação](#/docs/00-architecture/verification.md) |
| 13 | Tools/Workspace/Acessibilidade/GUI | ⏸ discussão conjunta | `PLANNED` — sessão/input mínimos presentes, mas **fora do fechamento**; retomar em discussão conjunta | `crates/petunia-ui/src/app.rs`, `input.rs` |

## Crates — arquivos presentes vs. alvo

| Crate | Arquivos presentes (2026-10-09) | Alvo da matriz | Leitura |
|---|---|---|---|
| `petunia-core` | `color, document, error, id, lib, math, path, scene, units` | ~20 módulos (`shape, generated, paint, appearance, effects, text, raster, resources, styles, symbols, guides, serialization, error`…) | `PARTIAL` inicial |
| `petunia-engine` | `command, error, lib, snapping` | `command/, geometry/, spatial/, brush/, raster/, text/, layout/, color/, import/, export/, persistence/, fragments/, jobs/, plugins/` | `PARTIAL` inicial |
| `petunia-render` | `backend, error, lib, software` | `backend, frame, graph, vector/, raster/, text/, compositor/, effects/, cache/, overlay/, output/, software` | referência mínima |
| `petunia-ui` | `app, error, input, lib` | `app, session, input, actions, tools/, viewport/, panels/, workspace/, accessibility/` + borda Qt/QML | sessão mínima; GUI fora do fechamento |

## Legenda

| Estado | Significado nesta página |
|---|---|
| `SPECIFIED` | Contrato escrito com invariantes e gates. Não significa código pronto. |
| `PARTIAL` | Código incompleto no checkout; faltam tipos/fluxos do contrato. |
| `PLANNED` | Item no roadmap/matriz, ainda não codificado. |
| `TESTED` / `VERIFIED` | Somente com comando executado + resultado identificável. Nenhuma etapa reivindica isso aqui. |
| `UNVERIFIED` | Sem fonte ou execução suficiente para afirmar. |

## Atualizar esta página

1. Execute a verificação (`ls`, `cargo test`, benchmark ou fuzz) e anote comando + saída.
2. Mude apenas a linha da etapa afetada, citando caminho/arquivo e, quando houver, commit.
3. Se o código divergir da spec, registre **implementation gap** com as duas fontes — não reescreva a spec silenciosamente.
4. Mudança de decisão fechada exige ADR novo ou supersession, conforme [leitura e autoridade](#/docs/07-agents/authority-reading.md).

Referências: [matriz de implementação](#/docs/00-architecture/implementation-matrix.md) · [capacidades](#/docs/00-roadmap/capabilities.md) · [verificação e gates](#/docs/00-architecture/verification.md) · [workflow](#/docs/07-agents/implementation-workflow.md).
