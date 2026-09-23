# 09.22 — Cargo Workspace, Crate Topology, Dependency Direction & Feature Boundaries

# Goal

Translate architectural boundaries into enforceable Rust workspace structure without creating dozens of empty crates prematurely.

# Layers

Recommended logical layers:

1. `foundation`: IDs, diagnostics, units/math primitives, schemas, resource IDs;
2. `domain`: document, geometry contracts, color semantics, text semantics, raster semantics;
3. `application`: actions, commands, history, jobs, capability registries, sessions;
4. `engines`: geometry adapters, raster engine, evaluation, render/compositor, typography/color implementations;
5. `io`: native format, SVG/PDF/raster import/export;
6. `extension`: plugin host, MCP, resource packs;
7. `platform`: OS services;
8. `ui`: Slint adapter/design system/personas/panels;
9. `apps`: desktop binary, CLI/headless/testing binaries.

# Dependency rule

Dependencies flow inward/downward toward smaller contracts. `domain` never depends on `application`, engines, IO, extension, platform or UI. UI/IO/plugins call application/domain contracts, not each other.

# Candidate crates

Canonical crate/binary naming now follows the Petunia Design Studio product identity. If an implementation branch still contains legacy `vd_*` names inherited from VectorVonDoom, treat them as temporary migration aliases and do not expose them as new public APIs.

`petunia_design_ids`, `petunia_design_diagnostics`, `petunia_design_schema`, `petunia_design_resources`, `petunia_design_document`, `petunia_design_color`, `petunia_design_text`, `petunia_design_raster_model`, `petunia_design_geometry`, `petunia_design_actions`, `petunia_design_commands`, `petunia_design_history`, `petunia_design_capabilities`, `petunia_design_jobs`, `petunia_design_evaluation`, `petunia_design_scene`, `petunia_design_vector_engine`, `petunia_design_raster_engine`, `petunia_design_compositor`, `petunia_design_typography`, `petunia_design_color_engine`, `petunia_design_native`, `petunia_design_svg`, `petunia_design_pdf`, `petunia_design_image_io`, `petunia_design_plugin_api`, `petunia_design_plugin_host`, `petunia_design_mcp`, `petunia_design_platform`, `petunia_design_resource_packs`, `petunia_design_i18n`, `petunia_design_config`, `petunia_design_ui_slint`, `petunia-design-desktop`, `petunia-design-cli`.

This is a **logical map**; combine small crates until independent compilation/ownership/testing justifies separation.

# Foundational vs optional

Foundational crates cannot be runtime-disabled because they define canonical contracts (`petunia_design_document`, IDs, commands). Product capabilities such as Data Merge, Photo filters, specific importers or specialty panels may be modules contributed through registries even if compiled into the binary.

# Cargo features

Use features for platform/build adapters or heavyweight optional dependencies, not user-facing enable/disable state. Runtime capability registry controls user modules. Avoid feature combinations that alter document semantics.

# Enforcement

Use dependency graph checks (`cargo metadata`, architecture test script), deny forbidden dependency edges in CI, `pub(crate)` by default for internals, narrow facade traits, and avoid global mutable singletons.

# Compile-time ergonomics

Generated constants for built-in namespaced IDs can live in a generated/resources crate. Runtime registries accept arbitrary valid IDs from plugins/resource packs.

# Tests

Headless core compiles without Slint/wgpu where possible; `petunia-design-cli` exercises document/import/export; minimal-feature CI catches accidental UI/platform dependency leakage.

# Initial physical workspace recommendation

The logical map above is intentionally finer than the first physical Cargo workspace. V1 should begin with a **moderate crate count** and split only at real dependency/security/compile/test boundaries.

Recommended starting physical grouping:

```
crates/
  petunia_design_foundation/       # IDs, units, schema primitives, diagnostics contracts
  petunia_design_document/         # canonical document + semantic resource/object model
  petunia_design_application/      # Actions, Commands, history, sessions, capabilities
  petunia_design_jobs/             # executor-neutral job contracts/scheduler
  petunia_design_geometry/         # Petunia Design Studio geometry API + adapters internally
  petunia_design_raster/           # pixel model + CPU raster/brush contracts
  petunia_design_text/             # text semantic model + layout facade
  petunia_design_color/            # semantic color + CMM facade
  petunia_design_evaluation/       # derived graph/cache/invalidation
  petunia_design_render/           # scene + vector/raster compositor facades
  petunia_design_io/               # native/SVG/PDF/image adapter registry; submodules initially
  petunia_design_resources/        # resource packs, i18n, typed settings/resources
  petunia_design_extension/        # plugin API/host + Lua binding + optional WASM feature/subcrate
  petunia_design_mcp/              # MCP adapter/protocol surface
  petunia_design_platform/         # typed OS service ports + platform adapters
  petunia_design_ui_slint/          # Slint shell/design system only
apps/
  petunia-design-desktop/
  petunia-design-cli/
  petunia-design-conformance/
```

As a subsystem grows or requires a heavy/unsafe dependency boundary, split it into the finer candidate crates already listed. Do not pre-create empty crates solely to match diagrams.

# Crate split criteria

Split a module into an independent crate when at least one is true:

- dependency would otherwise leak a heavyweight/unsafe/native library into unrelated builds;
- independent compilation/testing/security ownership is valuable;
- it defines a stable public contract used by several peers;
- it needs feature-gated platform/backend implementations;
- compile times/change frequency materially improve through isolation;
- licensing/distribution profile differs (e.g. optional Wasmtime tier).

Do **not** split merely because a Rust module is conceptually named in documentation.

# Strict dependency DAG

Canonical high-level dependency DAG:

```
foundation
   ↑
document/domain semantics
   ↑
application/contracts
   ↑
engines / IO / resources / extension / MCP / platform adapters
   ↑
UI + apps
```

Lateral peers cooperate through lower-layer contracts/registries; they do not depend on each other's concrete implementation. `petunia_design_io` may use domain/evaluation/render export ports, but `petunia_design_document` never imports `petunia_design_io`.

# Forbidden examples

CI must reject patterns such as:

- `petunia_design_document -> gpui`;
- `petunia_design_document -> wgpu/vello`;
- `petunia_design_geometry -> petunia_design_ui_slint`;
- `petunia_design_color -> krilla`;
- `petunia_design_plugin_api -> mlua` (semantic API must remain runtime-neutral);
- `petunia_design_application -> petunia_design_ui_slint`;
- Design feature crate directly importing Photo feature implementation;
- one importer directly calling another importer's private parser.

# Third-party dependency containment

Third-party types should terminate at adapter boundaries. Prefer wrapping/re-exporting **Petunia Design Studio-owned semantic types**, not public third-party structs.

Examples:

- Kurbo path/affine types converted at `petunia_design_geometry` adapter boundary;
- Vello Scene/Brush types stay in render backend;
- `mlua::Lua/Value` stay in Lua binding/host crate;
- Slint Entity/Element stay in `petunia_design_ui_slint`;
- LittleCMS handles stay in color-engine adapter;
- ZIP/parser implementation types stay inside native IO.

A dependency replacement should generally affect one adapter crate/module, not the full codebase.

# `unsafe` policy

Default policy: `unsafe` is forbidden in pure foundation/domain/application crates unless an ADR proves necessity. Unsafe code is isolated into the smallest adapter/backend module/crate with:

- documented safety invariants;
- dedicated tests/fuzzing;
- code-review owner/rationale;
- no unsafe value escaping as unchecked domain state.

Prefer `#![forbid(unsafe_code)]` in crates that need none and explicit lint exceptions only where justified.

# Public API policy

Use `pub(crate)`/private by default. A symbol becomes public across crates only when it is part of a deliberate port/semantic contract.

Public structs should avoid fields that force downstream construction against unstable internals. Prefer constructors/builders/typed DTOs where invariants matter. Avoid `pub` fields containing third-party types.

# Error policy across crates

Cross-crate semantic contracts return Petunia Design Studio-owned typed errors/diagnostics. `anyhow`-style opaque context may be useful in binary/tools/adapters internally but is not the stable domain API. Preserve source errors for diagnostics without exposing dependency-specific error types across architectural boundaries.

# Async policy

Do not mark every service `async` because Tokio exists. Domain calculations stay synchronous/pure where natural. Async appears at I/O/job/application orchestration boundaries. No domain type requires a Tokio runtime to be constructed/tested.

# Feature flag policy

Allowed Cargo feature categories:

- platform target/backend selection;
- optional heavyweight adapters (`wasm-plugins`, optional codecs/backends);
- developer/test/profiling instrumentation;
- build profile aggregation (`headless`, `minimal`).

Forbidden feature behavior:

- changing canonical document meaning;
- silently changing command validation semantics;
- user preference/module enable state;
- creating two incompatible serialized schemas from one version.

# Build profiles / supported matrices

Define named supported workspace profiles rather than testing arbitrary powerset combinations:

1. `desktop-default` — Slint + render + standard IO + Lua plugins;
2. `headless` — no Slint/windowing; core/app/IO/MCP/conformance as required;
3. `minimal-core` — document/commands/schema/native format tests;
4. `gauntlet` — extra diagnostics/fuzz/test-support;
5. `desktop-wasm-plugins` — optional Wasmtime tier when implemented.

CI guarantees these profiles. Unsupported ad-hoc feature combinations are not a compatibility promise.

# Architecture dependency test

Create a repository script/tool reading `cargo metadata` and a checked-in layer policy file. CI fails on forbidden edges/cycles. The test reports a path such as:

```
petunia_design_document -> petunia_design_render -> wgpu
```

with allowed-layer rule and remediation hint.

Also scan source/imports for especially prohibited framework crates in core layers as defense in depth.

# Crate ownership documentation

Every crate README/module root documents:

- responsibility/non-responsibilities;
- allowed dependency layers;
- public ports;
- thread/concurrency assumptions;
- persisted/public compatibility impact;
- primary tests/benchmarks;
- unsafe/FFI policy;
- canonical Atlas pages.

This is part of agent microcontext retrieval.

# Naming rules

Use full descriptive names in code. Avoid ambiguous abbreviations (`mgr`, `ctx`, `obj`, `cfg`) in public types/functions unless the term is a universally recognized domain acronym (ICC, PDF, SVG, GPU, MCP, ID). Local loop variables may remain idiomatic when scope is tiny, but public APIs prioritize clarity.

# Cyclic dependency resolution

When two modules appear to require each other, move the shared semantic contract inward or introduce an application-level coordinator. Do not solve Cargo cycles by creating an arbitrary `common` crate that becomes a dumping ground.

# Generated code/resources

Generated built-in ID constants/schema bindings live under clearly marked generated modules/artifacts and are reproducible from canonical source. CI regenerates/checks diff. Generated code must not be manually edited as the source of truth.

# Test placement

- unit tests live near pure logic;
- cross-crate contract tests live in owning contract crate or conformance app;
- importer/parser fuzz targets live near boundary adapter;
- headless end-to-end flows live in `petunia-design-conformance`/integration tests;
- Slint visual/semantic tests live only in UI crate/app.

Core test success must not require GPU/window server.

# Dependency-update rule

Before major dependency upgrade, identify adapter boundary, compatibility risk and gauntlets. Slint/Vello/wgpu/mlua/parser upgrades cannot be merged solely because `cargo update` compiles; run subsystem conformance and document behavior/API changes.

# Code-agent implementation rule

Before adding a dependency or crate, an agent must state:

1. which architectural layer owns it;
2. why an existing dependency/port is insufficient;
3. whether its types cross public boundaries;
4. optional/heavy/native/unsafe implications;
5. headless/minimal-profile effect;
6. license/SBOM impact;
7. removal/replacement path.

A dependency added without this classification is incomplete work.