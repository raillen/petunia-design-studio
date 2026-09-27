# 10.13 — Functional Gauntlet, Edge-Case Matrix & Tool Definition-of-Done

# Every tool gets a state-machine spec

For each interactive tool document: activation/deactivation, cursor, hit targets, pointer-down/move/up, keyboard modifiers, Esc/Enter behavior, context toolbar, Properties integration, transaction start/commit/cancel, selection effects, snapping, viewport auto-pan, invalid target behavior and automation equivalent.

# Every operation gets semantic contracts

Inputs, outputs, preconditions, resource/style inheritance, ID creation/deletion, undo, serialization, ChangeSet/invalidation class, deterministic behavior and failure diagnostic.

# Cross-cutting edge matrix

Test tools under: nested transforms, clipping/masks, locked/hidden objects, live effects, symbols, very large/small coordinates, multi-selection, alternate color models, high DPI/zoom extremes, missing resources/fonts, huge docs, alternate locale/keymap, cancellation and save/reopen.

# Property-based tests

Geometry/math operations use invariants; commands use undo roundtrips; data merge uses deterministic record generation; raster uses tile-boundary equivalence; text uses Unicode corpus.

# Interaction replay

Record semantic pointer/key/tool actions into replay fixtures where practical. Replays should produce stable document semantic snapshots even if render pixels vary slightly by platform.

# Definition of Done

No tool is complete with only a UI demo. It needs documented semantics, core command, history, persistence impact, diagnostics, tests, performance target, accessibility/action metadata and MCP/plugin schema where exposed.

# Cross-layer Definition of Done

A tool is not complete if its semantic behavior exists only inside GPUI event handlers. Tool logic must be testable through toolkit-neutral normalized input/application contracts and preserve the core↔UI boundary in 09.27.

For detachable/optional tools, Definition of Done additionally requires:

- capability manifest/contribution registration;
- clean disable/unload behavior;
- no direct concrete dependency from unrelated features;
- missing-tool workspace/document recovery;
- no orphaned jobs/listeners/actions;
- plugin/MCP discovery parity where exposed.

# UX and presentation Definition of Done

A visible tool must also satisfy 08.20 and 08.21: discoverable primary workflow, predictable Esc/Enter/undo behavior, all semantic UI states, tokenized copy/icons/styling/help/a11y metadata, localization expansion and semantic accessibility/inspection hooks.

# Documentation Definition of Done

Stable tools require English implementation/user documentation plus pt-BR translation, VitePress publication/build validation and updated generated Action/Property/tool catalogs. A code-agent implementation with passing code but stale docs is incomplete.

# Canonical functional specification template

Every new tool/operation page or section must include the following fields when applicable. A code agent may not mark a feature complete until the relevant fields are either specified or explicitly `Not Applicable` with rationale.

1. **Scope status:** `V1_REQUIRED`, `MILESTONE_REQUIRED`, `POST_V1_CANDIDATE`, `EXPERIMENTAL`, `OPEN_ADR`, `OUT_OF_SCOPE`.
2. **User outcome / job-to-be-done.**
3. **Activation / discoverability:** ActionId, ToolId, menus/palette/panel entry, TextId/IconId/help topic.
4. **Accepted targets and disabled reasons.**
5. **Canonical state machine.**
6. **Normalized pointer/keyboard/pen intents; no hard-coded toolkit events.**
7. **Selection effects and target ownership.**
8. **Preview/provisional state.**
9. **Transaction begin/update/commit/cancel.**
10. **Snapping/constraints/autopan.**
11. **Modifier/keymap intents.**
12. **Coordinate spaces/units/tolerances.**
13. **Created/deleted/modified IDs and resource/style inheritance.**
14. **Undo/coalescing/history label.**
15. **ChangeSet + invalidation classes.**
16. **Persistence/schema/migration impact.**
17. **Error/invalid-target/recovery behavior.**
18. **Concurrency/jobs/cancellation/stale-result policy.**
19. **Memory/performance budget and degradation policy.**
20. **Color/bit-depth/text/geometry semantics as applicable.**
21. **Accessibility and keyboard-only completion.**
22. **Localization/tokenization/pseudo-locale behavior.**
23. **Plugin capability/permission exposure.**
24. **MCP discovery/automation equivalent.**
25. **Security/threat inputs.**
26. **Diagnostics/observability.**
27. **Unit/property/fuzz/golden/replay/E2E tests.**
28. **English docs + pt-BR + VitePress/generated reference impact.**

# No implicit scope rule

Words such as `future`, `later`, `optional`, `planned`, `when supported`, `as engine matures` or `if implemented` are not valid scope statuses by themselves. Replace them with the formal taxonomy from 12.8/09.21. A UI control may only advertise capabilities whose functional specification status authorizes them for the current milestone.

# Functional authority rule

When documents conflict:

- Product Charter decides the product outcome/scope intent;
- accepted ADR/Architecture Atlas defines structural invariants;
- **Functional Engine Atlas defines user-visible semantic behavior**;
- Interface Atlas defines presentation/interaction realization without inventing new semantics.

A screenshot or existing code path cannot override these contracts.

# State-machine requirement

Interactive tools must model at least `Inactive/Idle`, active/provisional states, `Commit`, `Cancel` and invalid/suspended conditions where applicable. Tool switches, document close, module unload, target deletion, revision conflict and focus loss must have explicit behavior. “The GPUI handler stops receiving events” is never a valid lifecycle contract.

# Transaction and exact-cancel gate

For every continuous gesture, automated tests must prove that cancel returns canonical document state to the pre-gesture state, including IDs/resources created provisionally. Preview may change derived/display state but must not leak hidden history entries.

# Disabled-reason contract

Actions/tools/properties expose machine-readable disabled reason/category and optional recovery actions (`unlock`, `select PixelLayer`, `install capability`, `rasterize explicitly`, etc.). UI tooltips, MCP errors and plugin SDK introspection derive from the same reason metadata.

# Performance classes

Each feature declares a target class rather than one universal latency number:

- **Interactive continuous:** pointer/brush/transform feedback must target frame-budget responsiveness;
- **Interactive discrete:** commands should appear immediate or surface progress quickly;
- **Background visible:** cancelable progress with UI remaining responsive;
- **Batch/offline:** throughput/memory bounds dominate but cancellation and deterministic output remain required.

Benchmark baselines and p50/p95 are recorded on reference hardware profiles; code agents must not claim “fast enough” without measurements for performance-sensitive paths.

# Reference/oracle requirement

Numerically/visually complex operations require a semantic oracle before optimization: CPU/reference implementation, known fixtures, independent library comparison or mathematically defined invariant. GPU acceleration cannot become the only definition of correctness.

# Failure-injection matrix

At minimum test relevant features with: stale document revision, target deleted mid-operation, module/plugin disable, cancellation, allocation/resource-limit failure, GPU device loss where relevant, missing font/profile/resource, invalid numeric input, hostile imported data and save/close during background work.

# Modularity/detach gate

Optional functional modules must pass the 09.1 detach proof. Removing a tool/filter/importer/panel must remove discovery/UI contributions while leaving unrelated functionality operational and preserving opaque document data where required.

# Headless equivalence

Representative semantic workflows for every stable feature must be executable through toolkit-neutral Commands/Properties or normalized tool harness without GPUI. UI interaction tests add confidence but cannot be the only behavioral test.

# UI ↔ MCP ↔ Lua parity

When the same semantic operation is exposed through multiple surfaces, conformance tests compare canonical resulting state and diagnostics. Differences are allowed only for presentation/context, not business rules.

# Functional milestone audit

At milestone boundaries generate a coverage report containing all registered ActionIds, ToolIds, Effect/Adjustment kinds, Importer/Exporter IDs and major Property schemas mapped to:

- scope status;
- canonical functional section;
- core implementation owner;
- test fixture/gauntlet;
- Plugin/MCP exposure status;
- EN/pt-BR/VitePress documentation.

Unmapped stable registry items are release-blocking documentation drift.