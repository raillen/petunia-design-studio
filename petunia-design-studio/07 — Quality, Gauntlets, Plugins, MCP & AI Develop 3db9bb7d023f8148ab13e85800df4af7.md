# 07 — Quality, Gauntlets, Plugins, MCP & AI Development

# Quality architecture

A disciplina de qualidade do antigo VectorVonDoom é preservada, substituindo ferramentas C++ por equivalentes Rust.

- `cargo fmt` / `clippy` com warnings relevantes tratados como erro;
- unit + contract tests;
- `proptest` para invariantes;
- `cargo-fuzz` para geometry, SVG/native parsers, document mutations e hostile files;
- `insta`/goldens para serialization e outputs controlados;
- Criterion/perf budgets;
- coverage e CodeQL/Dependency review quando aplicável;
- dependency audit/licensing/SBOM policy;
- stress documents e memory/resource budgets.

# UI/visual testing herdado

Não depender apenas de screenshot. Cada tool/panel deve expor presentation state observável; canvas fornece test hooks em coordenadas de documento, hit targets e render state. Testar focus/tab order, input, keyboard contexts, docking, high-DPI, 100/125/150/200%, themes, empty/error/loading states e huge documents.

Fluxo E2E mínimo: create document → draw path/shape → node edit → apply semantic color → undo/redo → save/reopen → export.

# Differential gauntlets

## Geometry

Kurbo/iOverlay results comparados a reference implementations/fixtures quando possível; algebraic invariants, fuzzing e minimized regressions.

## Color

moxcms vs LittleCMS reference em perfis selecionados; medir diferenças perceptuais/numéricas e registrar fixtures.

## Rendering

CPU/headless reference e GPU goldens quando aplicável; separar tolerância de rasterização de mudança estrutural.

# MCP e automation

MCP não possui business logic paralela. Expõe Actions/Commands do mesmo registry da UI.

Exemplos: `create_surface`, `create_path`, `set_fill`, `boolean_union`, `add_adjustment`, `bind_data_field`, `preview_data_record`, `export_pdf`.

Adicionar Inspection API para UI tree/accessibility tree, focused state, active tool, document summary, viewport transforms e semantic hit targets para agentes executarem gauntlet loops na aplicação real.

# Plugins

Aubrieta defines one **runtime-neutral semantic Plugin SDK + capability broker**. **Lua 5.5 via `mlua` is the accepted V1 approachable scripting runtime**; Wasmtime/WASI is the high-isolation component tier. Python and JavaScript remain external automation languages through MCP/client SDKs unless a later ADR adds another embedded runtime. Every runtime/tier must expose the same semantic Actions/Commands/properties/contributions rather than parallel business logic.

Plugins V1 may register actions, effects/generators, importers/exporters, data-source adapters and declarative panels/tools where the relevant contribution contract is implemented. Any contribution class not implemented in the V1 milestone must be marked **Post-V1 Candidate** in the Plugin SDK capability catalog rather than described only as “future”. Network/filesystem/document write são capabilities explícitas, deny-by-default, not ambient runtime powers.

# AI-assisted implementation

Código gerado por agentes recebe exatamente os mesmos contracts, tests, performance budgets e security gates. Cada feature deve declarar acceptance criteria, invariants, failure modes, observability e rollback/recovery antes de ser considerada completa.

# Modularity and extension quality gates

Built-in modules must be tested as if they were detachable extensions. CI should include configurations with optional capabilities disabled, dependency-order randomized where deterministic, missing-provider fallbacks exercised, resource-pack corruption injected, plugin version mismatch tested and feature registration collisions rejected deterministically.

A feature is not complete until it declares: provided/required capabilities, contribution IDs, persistence keys, token/text/icon IDs, cancellation behavior for background work, permissions, observability hooks and removal/disable behavior.

# Code-agent documentation and implementation gates

The canonical operating protocol now lives in **12 — Code Agent Implementation Handbook & Living Documentation**. Agents must build a task microcontext from authoritative pages before coding, classify ambiguities, implement through headless semantic contracts first where possible, run gauntlet loops and update documentation as part of the same change.

## Documentation requirements

Behavior/API/UI changes require:

- canonical English repository documentation (`en-US`) first;
- synchronized Brazilian Portuguese (`pt-BR`) translation before release;
- VitePress build/link/reference checks;
- generated registry/API documentation refreshed where applicable;
- tested examples for plugin/MCP/public APIs;
- Atlas/ADR updates when the semantic contract changes.

Documentation drift is a quality failure, not editorial debt deferred indefinitely.

# Architecture quality additions

Quality gauntlets must now include:

- headless core execution without GPUI;
- forbidden dependency-edge checks;
- optional-module detach/unload proof;
- UI no-hardcode/tokenization checks;
- usability acceptance scenarios from 08.20;
- Design System portability rules from 08.21;
- plugin permission/rollback/quota tests from 09.28;
- MCP revision/transaction/discovery parity from 09.29;
- code-agent retrieval tests proving one unambiguous canonical implementation path can be found from documentation.