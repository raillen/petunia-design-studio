# 09.0 — Documentation Gap Audit & Completeness Matrix

<aside>
🔎

**Audit finding:** the notebook had strong product direction and an unusually complete UI specification, but most non-UI domains were still one or two abstraction levels above implementation. This matrix is the remediation checklist.

</aside>

# Completeness standard

A domain reaches **Implementation-grade** only when it documents data model, contracts/APIs, lifecycle/state machine, error/failure behavior, concurrency, persistence/migration, performance/memory, extension points, security, observability, tests and staged implementation.

| Domain | Before audit | Primary gap | Canonical remediation |
| --- | --- | --- | --- |
| Modularity/extensions | Conceptual | no runtime capability/contribution lifecycle | 09.1 |
| Document/object/resource model | Partial | object graph/invariants/unknown data not formalized | 09.2 |
| Commands/undo | Conceptual | transactions, ChangeSets, coalescing, large raster undo | 09.3 |
| Evaluation/caches | Conceptual | dependency graph and invalidation granularity | 09.4 |
| Vector geometry | Feature list | numeric/tolerance/robustness contracts | 09.5 + Functional Atlas |
| Raster/Photo | Feature list | tile lifecycle, brush/selection/undo internals | 09.6 + Functional Atlas |
| Rendering/compositor | Stack choice | scene model, blend/isolation, GPU lifecycle/device loss | 09.7 |
| Typography/layout | Stack choice | canonical text model, font resolution, editing/flow | 09.8 + Functional Atlas |
| Color management | Conceptual | profile lifecycle, transforms, command semantics | 09.9 |
| Native persistence | Very shallow | schema/version/migration/atomic-save/recovery | 09.10 |
| Import/export | Feature list/UI-heavy | adapter contracts, degradation negotiation/security | 09.11 |
| Themes/i18n/icons | UI tokens partial | resource pack schemas/resolution/extensibility | 09.12 + 09.16 |
| Jobs/concurrency | Incidental | priorities, cancellation, document mutation ownership | 09.13 |
| Plugin architecture | Conceptual | component interfaces, versioning, lifecycle, permissions | 09.14 |
| MCP/automation | Conceptual | schema/versioning, atomicity, subscriptions, inspection | 09.15 |
| Platform services | Missing | OS-specific boundaries and abstractions | 09.17 |
| Security | Scattered | single trust model/threat boundaries | 09.18 |
| Observability | Scattered | structured tracing/diagnostics/crash bundles | 09.19 |
| Build/release | Missing | packaging/signing/update/SBOM/reproducibility | 09.20 |
| Architecture governance | Partial | ADRs/compatibility/DoD/no-gap gates | 09.21 |
| Crate topology/dependency enforcement | Scattered | logical modules not mapped to enforceable Rust dependencies | 09.22 |
| Preferences/workspace persistence | UI-heavy | state scopes, precedence and config failure semantics missing | 09.23 |
| Application/document lifecycle | Partial | startup/open/multi-window/close/shutdown ownership not formalized | 09.24 |
| Property/parameter schemas | Missing | no shared reflection-free contract for inspectors/MCP/Data Merge/plugins | 09.25 |
| Explicit unresolved decisions | Scattered `later`/ADR notes | risk of forgotten decisions or premature guesses | 09.26 |

# Repeat-audit rule

Before each milestone, re-run this matrix and add new rows for any newly introduced subsystem. A domain cannot remain `TBD` solely because its UI has been implemented.

# Audit V2 — Code Agents, UI portability and living documentation

The second audit treats the notebook as an **implementation control plane for code agents**, not only a design reference. The following gaps were added to the canonical remediation map.

| Domain | Gap discovered | Canonical remediation |
| --- | --- | --- |
| UX/usability | visual/state specification was strong, but learnability, information architecture, cognitive load, user task metrics and structured usability testing were not explicit enough | 08.20 |
| Design System governance | token policy existed, but no complete enforcement contract for every presentation resource or future shell replacement | 08.21 |
| UI-agnostic core | GUI boundary was stated but lacked explicit ports, presentation-model rules, headless proof and shell-conformance tests | 09.27 |
| Plugin developer experience | Wasm safety architecture existed but API ergonomics, simple/advanced layers, permission UX, runtime neutrality and beginner documentation were underspecified | 09.28 |
| MCP developer/agent experience | automation architecture existed but lacked task-oriented helpers, dry-run/explain, context-efficient discovery, idempotency/retry rules and documentation cookbook standards | 09.29 |
| Code-agent operating protocol | agents had quality expectations but no canonical read order, microcontext packet, ambiguity classes, gauntlet loop or handoff contract | 12.2 + 12.4 + 12.7 |
| Living documentation | no repository/VitePress architecture, English-primary policy, pt-BR freshness contract, generated-reference policy or docs CI/release gate | 12.1 + 12.3 + 12.5 + 12.6 |

# Updated Implementation-grade standard

A subsystem is no longer Implementation-grade merely because its engine contract is detailed. When applicable it must also document **agent retrieval path, modular detach behavior, UI portability, resource tokenization, usability acceptance scenarios, plugin/MCP parity, English/pt-BR documentation and VitePress publication impact**.

# Audit enforcement

Future milestone audits must query not only `TBD`/missing subsystems but also ambiguity and duplication: two pages describing the same contract with different names, raw UI constants, direct toolkit dependencies, undocumented registry IDs, plugin/MCP methods without examples, stale translations and behavior that exists only in UI code.

# Audit V3 — Page-by-page code-agent conformance (2026-09-15)

The third audit reviewed the notebook page-by-page as an implementation guide rather than only a subsystem checklist. The detailed ledger is maintained in [13 — Full Notebook Page-by-Page Audit & Conformance Ledger](13%20%E2%80%94%20Full%20Notebook%20Page-by-Page%20Audit%20&%20Conformanc%203db9bb7d023f810fb6fdf8aa804ef74e.md).

## Major V3 remediations

- Functional Atlas pages were expanded from feature lists into state/transaction/error/performance/automation contracts.
- One canonical structural tree was frozen; “Layers” cannot become a parallel storage hierarchy.
- Photo scope was reconciled: Clone/Heal/content-aware retouching are `POST_V1_CANDIDATE`, not V1 placeholders.
- Raster active selection was classified as transient session editing state until explicitly materialized as a mask/channel.
- Data Merge V1 was frozen around CSV/TSV/JSON + safe declarative formatters/streaming; network sources and arbitrary expressions are Post-V1.
- Persona switching was frozen as presentation/workspace state with no document mutation/dirty history.
- Functional 10.13 now defines a mandatory 28-field feature specification template and UI↔MCP↔Lua/headless parity gate.
- Lua 5.5 + mlua wording was normalized to `ACCEPTED_V1`; stale “recommendation/open runtime ADR” language was removed from canonical runtime pages.
- A dedicated Prumo workflow page was added: installed CLI only, no Aubrieta dependency on Prumo source/runtime, Goal→Wave→dossier→microcontext→gauntlet→docs→handoff, with first Wave continuing through the documented MVP cut for new major subsystems.
- Naming/brand identifiers and migration discipline were formalized in section 11.
- The lexical audit now treats bare `future/later/planned/TBD` product statements as defects unless they are examples, ordinary temporal prose, or accompanied by a formal scope status.

## V3 completeness conclusion

At documentation level, no known Critical/High contradiction remains across the canonical UI/Architecture/Functional/Agent contracts. Remaining High-risk items are primarily **implementation evidence**: fuzzing, benchmarks, detach tests, shell conformance, plugin sandbox quotas, native-format migration/corruption fixtures, MCP parity and bilingual VitePress CI.

## Milestone rule

Before each milestone/release, agents must check the section-13 ledger for unresolved Critical/High findings relevant to their task. A new canonical page or major feature not mapped into that ledger is itself documentation drift.

# Audit V4 — Interaction depth and executable evidence — 2026-09-18

The documentation-gap model now includes two additional completion dimensions.

| Domain | Gap discovered | Canonical remediation |
| --- | --- | --- |
| Affinity-inspired tool UX | Persona/tool inventories existed, but many interaction details still depended on designer intuition or generic Functional Atlas semantics rather than a researched creative-tool reference model. | 08.22–08.29 + 08.31–08.33 |
| Cross-discipline workspace | Design/Photo Personas were clear, but there was no explicit contract for advanced mixed workspaces without semantic Persona coupling. | 08.28 |
| UI evidence | 08.16 defined gauntlets, but there was no dedicated canonical fixture/evidence architecture binding tool workflows to semantic snapshots, interaction traces and human-factors findings. | 08.30 + 14.7 |
| Implementation proof architecture | Residual risks were listed as future CI/benchmark/fuzz work but not organized as stable evidence IDs, corpus policy, hardware tiers, detach proof or deterministic AgentOps commands. | 14.1–14.8 |

A domain marked Implementation-grade must now be implementable **and provable**: its applicable section-14 evidence class must be identifiable before milestone completion.