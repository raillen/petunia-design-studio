# 13 — Full Notebook Page-by-Page Audit & Conformance Ledger

# Audit reset — 2026-09-21

O ledger anterior foi invalidado parcialmente pela mudança **Aubrieta Design → Petunia Design Studio**, **.aubrieta/.aubri → .PTND** e **GPUI → Slint**.

Toda página que dependa de identidade, crate/resource namespace, UI toolkit, file suffix, screenshots/goldens ou adapter contract volta a **Needs re-verification** até passar pelo novo Identity/Slint/Total-Assurance Gauntlet. Não reaproveitar score antigo como prova.

<aside>
🧾

**Purpose:** this is the exhaustive page-by-page conformance ledger for the Aubrieta Design notebook. It records whether every canonical page is implementation-grade for humans and code agents, what ambiguities or gaps were found, and what remediation was applied.

</aside>

# Audit rubric

Each page is reviewed against all applicable dimensions: **authority/scope**, **V1 vs post-V1 status**, **data/contracts**, **lifecycle/error behavior**, **modularity/dependency direction**, **UI-agnostic core boundary**, **UX/usability**, **tokenization/localization/accessibility**, **security/privacy**, **performance/resource budgets**, **tests/gauntlets**, **MCP/plugin parity**, **observability**, **migration/persistence**, and **documentation/code-agent usability**.

# Severity

- **Critical** — can cause incompatible architecture, data loss, unsafe behavior or a parallel source of truth.
- **High** — likely to make agents implement the right feature through the wrong boundary or leave a major user workflow underspecified.
- **Medium** — ambiguity, missing acceptance detail, incomplete status or missing cross-reference.
- **Low** — terminology, consistency, polish or navigation issue.

# Conformance states

- **Implementation-grade** — sufficient to implement without inventing architecture.
- **Implementation-grade with canonical references** — deliberately concise because deeper behavior is owned by referenced canonical pages.
- **Needs remediation** — material gap or ambiguity remains.
- **Historical / research** — informative, not normative.

# Audit rule

A page may be concise, but it must clearly state what it owns and where deeper contracts live. Duplication with divergent wording is worse than a short page with a precise canonical reference. Every remediation must preserve the single-source-of-truth hierarchy from section 12.

# Running ledger

The ledger is appended as sections while the audit proceeds. Final state will include every page under Aubrieta Design plus residual issues that require implementation evidence rather than documentation-only resolution.

# Audit snapshot — 2026-09-15

This pass treats the notebook as an **implementation control plane for code agents**. Documentation conformance does not claim that code already satisfies the contract; it means the page is sufficiently explicit to guide implementation or deliberately delegates detail to a named canonical page.

Legend used below:

- **IG** — Implementation-grade.
- **IG-REF** — Implementation-grade with canonical references; deliberately avoids duplicating deeper contracts.
- **HIST/REF** — historical/research/prior art only.
- **RUNTIME-EVIDENCE** — documentation is sufficient but release confidence still requires implementation/benchmark/gauntlet evidence.

# Root and foundation pages — audit ledger

| Page | State | Findings / remediation | Residual risk |
| --- | --- | --- | --- |
| Aubrieta Design root | IG-REF | Canonical product identity, `.aubrieta`/`.aubri`, GPUI-primary but UI-agnostic core, modularity, tokenization, Lua/mlua, Plugin/MCP and Atlas navigation aligned. | Keep generated navigation/links in sync as new canonical pages are added. |
| 00 — Product Charter & Scope | IG-REF | Expanded product outcomes, target jobs, V1/non-goals and formal scope taxonomy. Bare future/planned wording is non-authoritative. | Milestone plans must continue assigning formal status to newly proposed features. |
| 01 — Architecture, GUI Boundary & Canonical Document | IG-REF | Dependency direction corrected; canonical document separated from UI/render; bridge/ports point to 09.27. Single mutation path retained. | CI dependency graph must prove the boundary in actual crates. |
| 02 — Design Persona | IG-REF | Persona scope aligned with Functional Atlas. UI/workflow composition does not own document semantics. | New Design tools require 10.x spec before UI exposure. |
| 03 — Photo Persona | IG-REF | V1 raster foundation aligned with 10.9/10.10; deferred retouching no longer implied as baseline. | Runtime benchmarks must validate 16-bit/tiled interaction targets. |
| 04 — Canonical Rust Stack & Engine Boundaries | IG-REF | Library choices are adapter decisions behind Aubrieta contracts; GPUI, render, color, text, plugin runtime and file-format boundaries aligned. | Third-party versions remain changeable; dependency review/re-benchmark required on upgrades. |
| 05 — GUI Technology Research & Decision Matrix | HIST/REF | GPUI is clearly primary; alternatives retained as fallback/prior art rather than peer implementation targets. | Do not let old comparison language reopen the accepted UI decision without ADR evidence. |
| 05.1 — GPUI Primary UI Stack, Ecosystem & Icons | IG-REF | GPUI Kit/base layering, icon strategy, testing and extension boundaries aligned with Aubrieta Design System; plugin semantics no longer tied to GPUI. | Community GPUI crates remain optional/feature-gated until compatibility is proven. |
| 06 — Reference Projects | HIST/REF | Explicitly prior art: Adopt/Adapt/Avoid guidance cannot override canonical Atlas contracts. | Reference-project implementation details may age; never use as version authority. |
| 07 — Quality, Gauntlets, Plugins, MCP & AI Development | IG-REF | Headless core, detach proof, no-hardcode UI, usability, bilingual docs, VitePress and Code Agent Handbook gates integrated. | Actual CI scripts/benchmarks must eventually provide evidence for each gate. |

# Interface Atlas 08 — audit ledger

The UI Atlas is the most detailed presentation specification. Its authority is **presentation/interaction**, while Product Charter/Functional Atlas owns feature scope and Architecture Atlas owns semantic boundaries.

| Page | State | Primary conformance result |
| --- | --- | --- |
| 08 — Interface Atlas & Design System | IG | Canonical atlas/index, coverage rule and Apple-like restraint + Affinity-like workflow + Aubrieta identity established. |
| 08.1 — Visual Foundations | IG | Spacing, density, radii, type, color roles, motion, icons, shadows, themes and JSON/TOML token/resource policy specified; literals forbidden outside design-system definitions. |
| 08.2 — Application Shell | IG | Window anatomy, tabs, Personas, toolbars/tool rail/status, sizing/focus and semantic state separated from document state. |
| 08.3 — Docking | IG | Dock tree, splits, tabs, floating palettes, persistence, missing-panel recovery and module detach behavior defined. |
| 08.4 — Menus/Commands | IG | Single Action Registry, menus/submenus/context/palette/search/keymaps and semantic enable/disabled state defined. |
| 08.5 — Control Catalog | IG | Reusable controls, states, validation, focus/input, density/a11y/token ownership documented. |
| 08.6 — Canvas | IG | Viewport/rulers/guides/grid/selection/snapping/HUD/overlays/cursors connected to toolkit-neutral tool semantics. |
| 08.7 — Design Persona UI | IG-REF | Complete workspace/panels/tools presentation; feature availability defers to 10.x scope rather than inventing semantics. |
| 08.8 — Photo Persona UI | IG-REF | Photo panels/tools/analysis presentation aligned to 10.9/10.10; deferred Clone/Heal must not surface in V1. |
| 08.9 — File Workflows | IG | Welcome/New/Open/Place/Import/relink/missing assets/fonts/recovery flows and error states covered. |
| 08.10 — Export | IG | Quick/full/batch export, preview, presets, conflicts/progress and degradation/preflight interaction aligned to 09.11. |
| 08.11 — Variable Data UI | IG-REF | Sources/bindings/record preview/generation UI aligned to 10.11; deferred expressions/network sources cannot leak into V1 controls. |
| 08.12 — Preferences | IG | Themes/icons/density/workspace/keymaps/color/plugins/help and tokenized settings registry behavior specified. |
| 08.13 — Feedback/Modality | IG | Tooltip/popover/dialog/banner/toast/progress/error/empty/loading conventions and modality hierarchy specified. |
| 08.14 — Accessibility/Input | IG | Keyboard/focus/screen reader/pointer/pen/IME/localization/RTL/HiDPI/adaptive density treated as first-class contracts. |
| 08.15 — GPUI Implementation Map | IG-REF | Maps semantic Aubrieta components to GPUI/Kit/base without leaking toolkit types into core. |
| 08.16 — UI Gauntlet | IG | State matrix, shell/docking/tree/canvas/dialog regression, performance, a11y, localization, failure injection, token and portability gates; scope-drift rule added. |
| 08.17 — Panel Atlas | IG | Dockable inspector/browser inventory with ownership and empty/loading/error behavior. |
| 08.18 — Window/Dialog Atlas | IG-REF | Catalog no longer uses “planned” as scope authority; Functional/Product scope controls whether a dialog exists. About/updater placeholder ambiguity removed. |
| 08.19 — Interface Language | IG | Canonical naming/microcopy/units/errors/action wording and consistency rules. |
| 08.20 — UX/Usability | IG | Learnability, discoverability, cognitive load, JTBD, progressive disclosure, recovery and measurable usability signals formalized. |
| 08.21 — Design System Governance | IG | No-hardcode contract, semantic resources, component ownership, token enforcement and future shell portability defined. |

## Interface residual evidence

Documentation is implementation-grade, but release acceptance still requires the actual GPUI component gallery, accessibility tree tests, focus/keymap tests, visual regression corpus, 10k/100k virtualization benchmarks, mixed-DPI tests and usability sessions/replay evidence defined by 08.16/08.20.

# Architecture Atlas 09 — audit ledger

| Page | State | Primary conformance result / remediation |
| --- | --- | --- |
| 09 — Architecture & Implementation Atlas | IG | Defines required implementation dimensions and serves as architecture index. |
| 09.0 — Gap Audit | IG | Tracks original and Code-Agent-era gaps; implementation-grade now includes retrieval, detach, UI portability, Plugin/MCP and bilingual/VitePress impact. |
| 09.1 — Modularity/Capabilities | IG | Runtime capability/contribution model, extensible IDs, module manifests, no-sideways-calls, built-in≈extension principle, safe unload sequence and detach proof. |
| 09.2 — Document Model | IG | DocumentStore ownership, stable persistent IDs/runtime handles, one structural tree, references/resources, validation, unknown-data preservation and object-count/security limits specified. |
| 09.3 — Actions/Commands/Undo | IG | Single-writer mutation lane, revisions, command result taxonomy, transactional preview/commit/cancel, undo budgets/raster strategy, coalescing and ChangeSet publication clarified. |
| 09.4 — Evaluation/Cache | IG | Dependency/invalidation taxonomy, cache generations/state machine, stale-result rejection, memory policy and quality contexts made implementation-grade. |
| 09.5 — Vector Geometry | IG + RUNTIME-EVIDENCE | f64 canonical coordinates, operation-specific tolerances, booleans/strokes/snapping robustness, determinism and fuzz/differential requirements. |
| 09.6 — Raster Engine | IG + RUNTIME-EVIDENCE | 128×128 V1 logical tiles, 8/16-bit, tile lifecycle/storage, brush pipeline, masks/selections, undo/compression and CPU-reference/GPU acceleration boundary. |
| 09.7 — Render/Compositor | IG + RUNTIME-EVIDENCE | RenderScene separation, normative mask/clip/isolation/blend order, intermediate surfaces, Vello/wgpu boundary, device-generation/loss and partial-update contracts. |
| 09.8 — Typography Engine | IG + RUNTIME-EVIDENCE | TextStory/ranges, shaping/layout pipeline, font request/fallback identity, IME/BiDi, frames/flow, OpenType and export behavior formalized. |
| 09.9 — Color Management | IG + RUNTIME-EVIDENCE | ColorValue/context/profile lifecycle, moxcms primary + LCMS fallback/oracle, CMYK preserve-numbers, Spot/Registration, Assign vs Convert and proofing specified. |
| 09.10 — Native Format | IG | Open ZIP-compatible `.aubrieta`/`.aubri`, canonical JSON + binary resources, manifest/version/migration/atomic save/recovery/corruption handling and interoperability projections. |
| 09.10.1 — Open Format/Interchange | IG | Public schemas/spec strategy, PDF/SVG compatibility projections, direct universal-save modes, companions, clipboard and security/fidelity rules. |
| 09.11 — Import/Export Adapters | IG | Staged import, immutable export snapshots, capability/fidelity descriptors, degradation plan, hostile-input boundary and cancellation/atomic output. |
| 09.12 — Resource Packs/Tokens | IG | JSON runtime resources + TOML manifests, DTCG-oriented tokens, TextId/IconId, atomic snapshots/fallback/security and MF2 alignment; prior “message subset” ambiguity removed. |
| 09.13 — Job System | IG + RUNTIME-EVIDENCE | Execution domains, ownership, priority/fairness, admission/backpressure, cancellation hierarchy, stale policies, shutdown/device-loss behavior; cross-session resumable jobs explicitly Post-V1 Candidate. |
| 09.14 — Plugin Host | IG + RUNTIME-EVIDENCE | Lua 5.5/mlua accepted V1 scripting runtime; runtime-neutral SDK boundary, Wasmtime optional high-isolation tier, permissions/quotas/versioning/unload/update isolation specified. |
| 09.15 — MCP Architecture | IG | One automation model over semantic registries, revisions/transactions/jobs/inspection/security and parity gate; 09.29 owns ergonomics. |
| 09.16 — Localization | IG | No hard-coded UI strings, namespaced TextId, MF2 canonical dynamic syntax, ICU4X-compatible locale services, pseudo-locale/RTL; en-US source and pt-BR release requirement distinguished from docs translation. |
| 09.17 — Platform Services | IG | Filesystem/dialog/clipboard/drag-drop/fonts/monitors/pen/IME/URL/power boundaries, file grants and headless mocks defined. |
| 09.18 — Security | IG + RUNTIME-EVIDENCE | Unified trust model for files/resources/plugins/MCP, resource limits/path safety/unsafe/FFI/supply-chain/privacy/fuzzing and Lua capability sandbox constraints. |
| 09.19 — Observability | IG | Stable diagnostics, privacy/redaction classes, traces/correlation, performance metrics, developer inspectors and opt-in crash/support bundles. |
| 09.20 — Build/Release | IG + RUNTIME-EVIDENCE | Profiles/reproducibility/packages/signing/updater/SBOM/licenses; vendored Lua packaging and VitePress bilingual docs are release gates. |
| 09.21 — Governance/DoD | IG | ADR policy, compatibility dimensions, implementation order, Definition of Ready/Done, source authority, no-hidden-coupling and documentation drift rules. |
| 09.22 — Cargo Topology | IG + RUNTIME-EVIDENCE | Layered workspace/DAG, candidate crates, split criteria, features vs runtime modules, forbidden-edge CI, third-party/unsafe containment and headless profiles. |
| 09.23 — Preferences/Config | IG | Application/workspace/document/session/machine scopes, precedence, typed Setting registry, snapshots/atomic writes, previews, migration/invalid-manual-edit behavior and secrets separation. |
| 09.24 — App/Document Lifecycle | IG | ApplicationRuntime/DocumentSession/ViewSession ownership and state machines, per-view selection policy, open/save/close/multi-window/concurrent save/shutdown/recovery clarified. |
| 09.25 — Property Schema Registry | IG | Reflection-free typed PropertyValue, units/mixed/applicability/provenance/reset/editor hints/versioning, Data Merge/MCP/Lua projection and migrations. |
| 09.26 — ADR Registry | IG | Contradiction repaired: plugin scripting is no longer open; Lua/mlua recorded as `ACCEPTED_V1`. Resolved decisions retain revisit triggers. |
| 09.27 — UI-Agnostic Core | IG + RUNTIME-EVIDENCE | Ports/adapters, AubrietaGuiBridge, presentation models, normalized input, semantic dialogs, headless/mock-shell conformance and migration spike criteria. |
| 09.28 — Plugin SDK | IG + RUNTIME-EVIDENCE | Beginner-first runtime-neutral semantic SDK now explicitly bound to Lua for V1, declarative UI, permissions/brokers/quotas/transactions/versioning/testing/unload and agent-documentation rules. |
| 09.29 — MCP API Usability | IG + RUNTIME-EVIDENCE | Task helpers + generic primitives, compact discovery, revisions/atomic batches/dry-run/jobs/semantic inspection/errors/idempotency/pagination/privacy/cookbooks/conformance. |
| 09.30 — Scripting Runtime ADR | IG / ACCEPTED_V1 | Lua 5.5 + mlua accepted, safe profile recorded, Python/JS roles constrained, comparative gauntlet retained as evidence/regression rather than decision blocker. |

## Architecture residual evidence

No known documentation-level Critical/High ambiguity remains in the audited 09 contracts. The dominant residual risk is **implementation evidence**: benchmarks, fuzz corpora, differential color/geometry fixtures, package/install smoke tests, actual module-detach CI, shell-conformance tests, plugin quota/interrupt tests and MCP parity fixtures must prove the written contracts.

# Functional Engine Atlas 10 — audit ledger

| Page | State | Primary conformance result / remediation |
| --- | --- | --- |
| 10 — Functional Engine Atlas | IG | Functional behavior is authoritative for what tools/features do; architecture owns boundaries and UI Atlas owns presentation. |
| 10.1 — Selection/Transform/Align/Snapping | IG + RUNTIME-EVIDENCE | V1 scope, selection/transform state machine, coordinate spaces, key object, duplicate-drag, atomic transaction, snap service/hysteresis, semantic modifier intents and automation parity. |
| 10.2 — Pen/Pencil/Nodes/Knife/Scissors | IG + RUNTIME-EVIDENCE | Pen state machine, existing-path continuation, handle/node semantics, basic vs Smart Delete, pencil fitting pipeline, corner/knife/scissors, exact cancel and plugin/MCP surface. |
| 10.3 — Shapes/Boolean/Shape Builder | IG + RUNTIME-EVIDENCE | V1 parametric primitives formalized; specialty primitives Post-V1; typed parameter evaluation, live/baked boolean ownership, style inheritance, Shape Builder/Smart Fill transaction and offset/outline semantics. |
| 10.4 — Appearance/Fill/Stroke/Effects | IG + RUNTIME-EVIDENCE | V1 Appearance Stack, multiple fills/strokes, solid/gradient, stroke contract, opacity layers, baseline effects; pattern/image fill, advanced effects and animation explicitly Post-V1. |
| 10.5 — Layers/Masks/Symbols/Styles/Assets | IG + RUNTIME-EVIDENCE | One-tree rule removes parallel Layer hierarchy ambiguity; reparent/world-transform behavior, mask/clip, symbol overrides/cycles, style resources and cross-document asset remapping specified. |
| 10.6 — Typography Tools | IG + RUNTIME-EVIDENCE | Artistic Text/Text Frames/TextStory/flow/text-on-path/IME/BiDi/missing fonts/styles/core paragraph controls and Convert to Curves; advanced lists/hyphenation/long-doc explicitly Post-V1. |
| 10.7 — Surfaces/Layout | IG + RUNTIME-EVIDENCE | Unified Surface identity/order/coordinate spaces/resize/layout metadata/guides/bleed/lightweight SurfaceTemplate and 1000-Surface scaling; angled guides Post-V1. |
| 10.8 — Perspective/Warp | IG + RUNTIME-EVIDENCE | V1 projective transform, perspective grids/planes and baseline Warp/Envelope; modifier order, invalid/singular states, live vs baked placement, text/raster behavior and explicit conversion. |
| 10.9 — Photo Tools | IG + RUNTIME-EVIDENCE | V1 selections/brush/eraser/gradient/crop/8–16-bit; selection is transient session state until materialized; target safety and stroke pipeline explicit. Clone/Heal/content-aware retouching are Post-V1 and cannot appear in V1 discovery/UI. |
| 10.10 — Photo Adjustments/Filters/Analysis | IG + RUNTIME-EVIDENCE | V1 Levels/Curves/HSL/Exposure/White Balance + blur/sharpen/noise + histogram/channels/pixel inspector; typed nodes, color-space semantics, masks/live-vs-baked and plugin/MCP exposure. |
| 10.11 — Data Merge | IG + RUNTIME-EVIDENCE | V1 CSV/TSV/JSON, stable DataSource/Field/Record identity, parsing/security, typed bindings/formatters, isolated preview, immutable generation snapshot, preflight/streaming; SQLite/REST/Sheets/arbitrary expressions Post-V1. |
| 10.12 — Design ↔ Photo Interop | IG + RUNTIME-EVIDENCE | Persona switch explicitly presentation-only/no-dirty; shared applicability matrix, explicit Rasterize/Curves/Expand/Bake contracts, mixed vector+raster composition, ImageObject vs PixelLayer and history parity. |
| 10.13 — Functional Gauntlet/DoD | IG | Added canonical 28-field functional-spec template, formal status/authority rules, exact-cancel, disabled reasons, performance classes, oracle/failure-injection/modularity/headless/UI↔MCP↔Lua gates and milestone coverage report. |

# Brand page 11 — audit ledger

| Page | State | Primary conformance result |
| --- | --- | --- |
| 11 — Naming & Brand Audit | IG-REF | Aubrieta Design is canonical; `.aubrieta`/`.aubri`, format ID, `aubrieta.*` namespace, crate/CLI naming, historical-name prohibition, brand-resource separation, legal-clearance gate and rename migration discipline recorded. |

Brand/trademark clearance remains a **pre-release external legal/business gate**, not an unresolved implementation architecture decision.

# Code Agent Handbook 12 — audit ledger

| Page | State | Primary conformance result / remediation |
| --- | --- | --- |
| 12 — Handbook root | IG | Source hierarchy, EN canonical + pt-BR mandatory, executable docs, VitePress and Code-Agent contract; now explicitly requires installed Prumo CLI for substantial work. |
| 12.1 — Source of Truth | IG | Notion vs repository docs vs VitePress vs schemas/tests roles, conflict resolution, normative wording, translation freshness, glossary, ownership/drift/deprecation policy. |
| 12.2 — Agent Operating Protocol | IG | Required read order, microcontext, ambiguity classes, implementation sequence, clean-code directives, evidence-based gauntlet, completion report; Prumo-backed initialization and stop/continue discipline added. |
| 12.3 — Living Docs/VitePress | IG + RUNTIME-EVIDENCE | Repository IA, locales, frontmatter/versioning, generated references, tested examples, diagrams/search/CI/publishing/a11y/offline; Prumo output must land as reviewable VitePress-compatible source. |
| 12.4 — Plan/DoR/DoD | IG | Readiness questions, complete implementation-plan template, ADR triggers, feature/module/UI/API DoD; version-controlled implementation dossier requirement added. |
| 12.5 — API Docs Standard | IG | 15-part API entry, beginner-first docs, plugin/MCP/error/schema standards, generated-vs-handwritten boundary; Lua-first V1 Plugin SDK examples and context-efficient MCP requirements added. |
| 12.6 — Documentation Gauntlet | IG + RUNTIME-EVIDENCE | Coverage/correctness/retrievability/translation/a11y/navigation/version gates, drift injection/retrieval tests and evidence requirements for scores; docs performance/usability added. |
| 12.7 — Repository/Review Rules | IG | Change workflow, PR/commits/review/generated files/fixtures/dependencies/refactors/TODO/docs/handoff. Prumo remains external; handoff minimum is explicit. |
| 12.8 — Status Taxonomy | IG | Formal states replace vague future/later/planned wording; machine-readable status metadata and promotion rule added. |
| 12.9 — Change Impact Matrix | IG | Cross-cutting update matrix covers Actions/Commands/Properties/Tools/Panels/UI/tokens/text/icons/modules/permissions/MCP/file schema/import/export/jobs/settings/docs/dependencies/removal; Plugin SDK/config/Persona/brand and documentation-impact-manifest rows added. |
| 12.10 — Prumo CLI Workflow | IG | New canonical guide: installed CLI only, no Prumo source/runtime dependency, Goal→Wave→task→dossier→gauntlet→docs→handoff, MVP-through-first-Wave rule, microcontext/dossier/impact manifest, privacy/fallback and VitePress/translation gates. |

# Ledger page 13 self-audit

**State: IG as an audit/control document.** This page records documentation conformance, not implementation completion. It must be updated after new canonical pages, accepted ADRs or major scope changes. A new stable page absent from this ledger is itself a documentation-drift finding.

# Current residual-risk classification

## No known documentation Critical findings

No currently known canonical-page contradiction is expected to cause a parallel document truth, unsafe scripting runtime selection, GUI ownership leak or V1/Post-V1 scope inversion after this pass.

## High risk only if implementation diverges

The following are release-blocking **implementation-evidence** areas rather than missing prose:

- headless core + real GPUI shell conformance;
- module disable/unload/detach tests;
- Lua memory/CPU interruption and broker denial tests;
- native format migration/corruption/fuzz corpus;
- geometry/color/raster/render differential/reference tests;
- 8/16-bit Photo performance/memory benchmarks;
- accessibility/IME/BiDi/mixed-DPI UI tests;
- MCP revision/transaction/permission/parity fixtures;
- docs EN↔pt-BR freshness and VitePress/generated-reference CI;
- installer/signing/update/rollback/SBOM/reproducibility evidence.

## Medium/ongoing documentation risks

- Newly introduced features can reintroduce ambiguous scope words unless 12.8/10.13 CI/lint policy is implemented.
- Library/ecosystem research pages age faster than semantic contracts and require periodic freshness review.
- Legal/domain/trademark status is external and must be rechecked before public commercial release.

# Audit completion rule

This documentation pass is considered closed only after the final lexical/consistency sweep has no unexplained stale product/runtime decisions and the 09.0 audit matrix is updated to point to this ledger. Future milestones repeat the process incrementally rather than assuming this snapshot remains permanently complete.

# Audit V4 — Affinity Interaction Research & Evidence Architecture — 2026-09-18

This pass deepened the Interface Atlas against official Affinity Designer 2 / Affinity Photo 2 documentation and the September 2026 unified Affinity Studio model, then converted the remaining “implementation evidence” risk into a canonical section 14.

## New UI/UX canonical pages

- **08.22 — Affinity Reference Model:** IG-REF. Affinity is prior art, not product authority; Adopt/Adapt/Reject and scope-preservation rules prevent imitation from silently changing Aubrieta semantics.
- **08.23 — Tool Interaction Grammar:** IG. Normalized tool states, context-toolbar order, modifiers, HUD, handles, numeric takeover and cancel/commit behavior.
- **08.24 — Design Tool-by-Tool UX:** IG-REF. Detailed Move/Node/Pen/Pencil/Width/Corner/Contour/Knife/Shape Builder/Gradient/etc. interaction mapping, without promoting Post-V1 features.
- **08.25 — Design Panels UX:** IG. Layers/Properties/Appearance/Colour/Stroke/Transform/Assets/Symbols behavior and state coverage.
- **08.26 — Typography/Text UX:** IG-REF. On-canvas editing, text objects, style/variable-font/missing-font flows and international text coverage.
- **08.27 — Precision/Snapping:** IG. Global/local snap policy, snap provenance, guides, numeric expressions and spatial feedback.
- **08.28 — Personas/Workspace Profiles:** IG. Design/Photo remain canonical defaults; advanced users may compose mixed Workspace Profiles without document mutation.
- **08.29 — Workflow Completion:** IG-REF. Place/import/export/preflight/recovery as one continuation of editing.
- **08.30 — UI/UX Evidence Corpus:** IG. Semantic snapshots, visual goldens, interaction traces, usability evidence and performance fixtures.
- **08.31 — Photo Tool-by-Tool UX:** IG-REF for the formal V1 subset; detailed interaction exists for broader Affinity prior art without changing 10.9 scope.
- **08.32 — Photo Nondestructive Panels:** IG-REF. Adjustments, live filters, masks, channels, histogram/brush panels and analysis-job behavior.
- **08.33 — Affinity Coverage Ledger:** IG-REF. The official Designer/Photo tool and panel catalogs are explicitly mapped to Adopt/Adapt/Reference/Outside Current Product plus canonical Aubrieta authority, preventing both omissions and accidental scope inflation.

## New evidence atlas

[14 — Implementation Evidence, Verification & AgentOps Atlas](14%20%E2%80%94%20Implementation%20Evidence,%20Verification%20&%20Agent%203df9bb7d023f81a78feedf6410de5065.md) is now the canonical implementation-evidence layer. Its 14.1–14.8 pages convert previously residual high-risk items into explicit proof contracts: test/conformance matrix, hardware tiers/benchmarks, golden corpus, detach proof, fuzz/security, native-format torture suite, UI/UX evidence integration and deterministic AgentOps/CI.

## Scope protection

The research does **not** make current Affinity feature breadth Aubrieta V1 scope. RAW/Develop, Liquify, panorama/HDR, advanced retouching, ML selections and other capabilities remain governed solely by Product Charter, Functional Atlas and 12.8 statuses.

## Updated residual risk

No new documentation Critical/High contradiction was introduced by this pass. The remaining material risk is now whether implementation and CI actually produce the evidence required by section 14. Before an Aubrieta milestone is called implementation-ready, at minimum the repository must instantiate the evidence IDs/corpus, dependency/detach checks, representative performance tiers, semantic UI fixtures and migration/fuzz gates described there.