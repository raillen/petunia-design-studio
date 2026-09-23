# 12.2 — Code Agent Operating Protocol, Microcontexts, Gauntlet Loop & Decision Discipline

<aside>
🤖

Code agents are implementation participants, not autonomous sources of product truth. Their job is to execute the documented architecture, surface ambiguity and improve the documentation when implementation reveals missing contracts.

</aside>

# Required reading order

For every task, agents should load only the smallest authoritative context that still preserves correctness:

1. Aubrieta root charter;
2. relevant 08/09/10 Atlas index;
3. exact subsystem page(s);
4. 09.1 modularity + 09.21 governance when changing architecture;
5. 08.20/08.21 for visible UI;
6. 09.27 for core↔UI boundary;
7. 09.28/09.29 for plugin/MCP surfaces;
8. this handbook.

# Microcontext packet

Before coding, create an internal implementation packet containing:

- user goal/problem statement;
- canonical pages/ADRs consulted;
- affected crates/modules;
- required/provided capabilities;
- invariants/non-goals;
- public/persisted API impact;
- Action/Command mapping;
- Property/Text/Icon/Token IDs;
- threading/job implications;
- security/permission implications;
- accessibility/localization implications;
- tests/fixtures/benchmarks required;
- documentation files that must change.

The microcontext should be concise enough for an agent to reason reliably and regenerate from canonical sources.

# Ambiguity classes

**Local reversible detail:** agent may choose simplest maintainable option and document it in code/review notes.

**Cross-module/persisted/public/UI-semantic decision:** agent must not silently decide. Draft or update ADR/spec before treating choice as canonical.

**Contradiction:** stop extending the contradiction; identify sources and resolve/supersede explicitly.

# Implementation sequence

Preferred order:

1. contract/types;
2. headless semantic tests;
3. domain/application implementation;
4. error/diagnostic paths;
5. jobs/cancellation if needed;
6. UI adapter/presentation model;
7. accessibility/localization/tokens;
8. MCP/plugin exposure where relevant;
9. integration/E2E;
10. performance/security gauntlets;
11. documentation + translation.

UI-first prototypes are allowed only as disposable experiments and cannot become canonical business logic.

# Change isolation

Agents should make cohesive changes, avoiding opportunistic unrelated refactors. If a required refactor crosses module boundaries, state why and preserve dependency direction.

# Clean-code directives

- descriptive full names over ambiguous abbreviations;
- small modules with explicit responsibility;
- no hidden global mutable state;
- no duplicate business logic across UI/MCP/plugin adapters;
- prefer typed domain values over generic JSON blobs internally;
- isolate `unsafe`/FFI with safety contracts;
- comments explain why/invariants, not restate syntax;
- tests name behavior and failure condition.

# Gauntlet loop

After a meaningful vertical slice, score the implementation against fixed dimensions without inflating scores:

- correctness;
- architecture/modularity;
- tests/edge cases;
- performance/resource use;
- security;
- UI/UX/usability if visible;
- accessibility/localization;
- plugin/MCP parity if applicable;
- documentation/maintainability.

For each dimension record evidence, defects and next fixes. Repeat implementation → tests → audit until no material defect remains for the target milestone.

A score may stay below 10 when scope is intentionally incomplete. Never weaken tests or redefine acceptance criteria to manufacture a higher score.

# No-progress condition

If repeated loops do not improve a dimension, identify architectural cause, dependency limitation or deferred product decision. Convert it to explicit ADR/backlog risk rather than infinite cosmetic iteration.

# Verification before claiming completion

Agent must demonstrate:

- targeted tests pass;
- broader relevant suite passes;
- no forbidden dependency edge;
- headless/core path still works;
- UI has required states and tokens;
- permissions/security cases pass;
- docs EN + pt-BR updated;
- VitePress build/link checks pass;
- no unresolved TODO introduced into stable contract.

# Final implementation report

Summarize what changed, contracts affected, tests run, documentation updated, performance/security notes, known limitations and any deferred ADRs. Do not report a feature as complete when only the happy path exists.

# Prumo-backed task initialization

For any substantial feature, refactor, migration, subsystem or documentation wave, the code agent must use the **installed Prumo CLI** to organize/refresh the task documentation and implementation context before broad coding begins.

Rules:

- use the CLI as installed on the development machine/environment;
- inspect `prumo --version` / CLI help or equivalent supported discovery before relying on command syntax;
- do **not** read, vendor or depend on the Prumo source repository as part of Aubrieta implementation merely to use its CLI;
- Prumo output is subordinate to Aubrieta's accepted specifications/ADRs and must be reviewed for drift;
- generated/refactored documentation must remain ordinary repository Markdown/JSON/TOML and VitePress-compatible rather than becoming readable only through Prumo.

The dedicated workflow is specified in 12.10.

# Stop/continue discipline

Agents do not ask for approval for every reversible local choice. They must stop architectural improvisation when a decision would create/modify a persisted/public contract, security boundary, module dependency direction, user interaction paradigm or compatibility promise. In that case, update the relevant spec/ADR or record an Open ADR with safe interim behavior before continuing.

For implementation defects whose fix is clearly implied by accepted contracts, continue fixing and testing without manufacturing a new product decision.