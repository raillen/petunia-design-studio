# 12.10 — Prumo CLI Workflow, Goals/Waves, Implementation Dossiers, Microcontexts & Agent Handoffs

<aside>
🧭

**V1 development-process requirement:** substantial Aubrieta implementation work uses the **installed Prumo CLI** as a documentation/harness orchestration aid. Prumo does not become an Aubrieta runtime dependency, source dependency or alternate product authority.

</aside>

# Purpose

Prumo structures work so humans and code agents can move from canonical Aubrieta specifications to small, reviewable implementation contexts without dumping the entire notebook into every task. It helps organize Goals, Waves, task dossiers, Gauntlet Loops, documentation impact and handoffs.

# Hard boundary

- Use the **CLI installed in the development environment**.
- Before relying on a command/flag, inspect the installed CLI version and built-in help. This handbook intentionally does not freeze command names that may evolve independently.
- Do **not** clone/read/vendor the Prumo implementation repository merely to use the CLI.
- Do not import Prumo crates/packages into Aubrieta production code.
- Prumo-generated guidance is subordinate to accepted Aubrieta ADRs/Atlas contracts.
- Stable outputs must be ordinary version-controlled Markdown/JSON/TOML/schemas and remain useful without Prumo installed.

# Work hierarchy

Use this conceptual hierarchy regardless of the exact current Prumo command syntax:

```
Product / Milestone
  ↓
Goal
  ↓
Wave
  ↓
Work Package / Task
  ↓
Implementation Dossier + Microcontext
  ↓
Vertical Slice
  ↓
Gauntlet Loop
  ↓
Documentation / Translation / Generated Reference
  ↓
Handoff / Close
```

# Goal contract

A Goal records:

- stable Goal ID/title;
- user/system outcome;
- formal scope status and milestone;
- canonical Notion/ADR references;
- success metrics/acceptance evidence;
- non-goals;
- required capabilities/modules;
- known dependencies/blockers;
- security/data-compatibility considerations;
- UX/usability outcome when visible;
- completion evidence expected.

A Goal describes an outcome, not a list of filenames to edit.

# Wave contract

A Wave is a coherent implementation slice that leaves the repository in a testable state. It declares:

- parent Goal;
- starting assumptions and prerequisites;
- included work packages;
- contracts/schemas allowed to change;
- feature/module detach implications;
- tests/benchmarks/gauntlets;
- documentation impact;
- exit criteria.

The **first implementation wave for a new major subsystem should proceed through the documented MVP cut**, not stop after scaffolding, unless an accepted blocker/dependency prevents it.

# Microcontext packet

For each task, Prumo/the agent should assemble only the authoritative context needed to implement correctly. Include:

- goal/problem statement;
- scope status;
- exact canonical page/ADR IDs or repository doc paths;
- affected crates/modules and allowed dependency direction;
- required/provided capabilities;
- invariants and non-goals;
- data model/schema/public API impact;
- Actions/Commands/Properties/semantic resource IDs;
- lifecycle/threading/jobs/cancellation;
- security/permissions/trust boundaries;
- UI/UX/a11y/tokenization requirements;
- Plugin/MCP exposure;
- tests/performance budgets;
- documentation impact.

Exclude unrelated notebook history. Never use a giant context dump as a substitute for selecting authority.

# Implementation dossier

Every substantial task/wave maintains a version-controlled dossier with at least:

```
Status / scope
Goal and user outcome
Non-goals
Canonical references
Architecture/dependency map
Affected crates/modules
Capability contributions/dependencies
Data model / schemas / migrations
Actions / Commands / Transactions
Properties / resources / IDs
Jobs / concurrency / cancellation
Security / permissions / hostile inputs
UI presentation / UX / accessibility / localization
Plugin SDK / MCP impact
Performance and memory budgets
Test / fuzz / golden / replay plan
Documentation impact manifest
Implementation log / evidence
Known risks / Open ADRs
Final handoff / residual work
```

Dossiers explain **this implementation work**. They do not replace canonical Atlas pages.

# Documentation impact manifest

For each substantial task, explicitly list applicable rows from 12.9 and the actual artifacts touched. At minimum consider:

- canonical Atlas/ADR;
- EN repository docs;
- pt-BR mirror;
- VitePress nav/sidebar;
- Action/Property/Capability registries;
- TextId/IconId/token catalogs;
- Plugin/MCP generated reference;
- diagnostics catalog;
- native/config schema docs;
- user workflow/troubleshooting docs;
- changelog/migration notes.

`Not applicable` is acceptable when accompanied by a reason for normally expected artifacts.

# Prumo workflow — semantic sequence

Because CLI syntax can evolve, agents follow this sequence rather than inventing fixed commands:

1. inspect installed Prumo version/help and repository configuration;
2. identify/create the Goal/Wave/work package representation supported by that version;
3. link canonical Aubrieta references rather than copying them as new authority;
4. generate/update the task microcontext and implementation dossier;
5. implement the smallest complete vertical slice through domain/application first;
6. run targeted tests and the relevant fixed-score Gauntlet Loop;
7. remediate material findings and repeat without weakening criteria;
8. update English documentation first, then pt-BR;
9. regenerate API/catalog references and build VitePress;
10. record evidence, residual risks and handoff/close state.

# Generated vs handwritten content

Prumo/code agents may generate indexes, matrices, task packets and reference projections. Handwritten normative rationale must not be overwritten blindly.

Generated sections/files should be identifiable by path/frontmatter/header and reproducible from their source. If a generator disagrees with a canonical contract, fix the source/generator rather than manually patching generated output.

# Gauntlet integration

Prumo-organized Gauntlet Loops use the fixed Aubrieta dimensions from 12.2/07/08.16/10.13. Scores require evidence. A loop may stop below 10 only because of an explicitly accepted milestone scope, blocked dependency or Open ADR — never because acceptance criteria were silently weakened.

For new language/runtime/engine foundations, the first Wave continues until the documented MVP cut is working end-to-end and its target gauntlets pass.

# Handoff contract

Every handoff records:

- Goal/Wave/work-package identity;
- scope status;
- canonical references used;
- exact current implementation state;
- changed modules/contracts/schemas;
- tests/gauntlets and evidence;
- EN/pt-BR/VitePress/generated-reference status;
- compatibility/migration/security notes;
- unresolved risks/ADRs;
- exact next safe step.

Never rely on chat history, hidden chain-of-thought or undocumented agent memory as the only continuation mechanism.

# Failure/fallback behavior

Prumo being unavailable or temporarily incompatible with an environment **must not block Aubrieta product correctness**. Use the same version-controlled dossier/microcontext/impact-manifest structure manually, note the tooling limitation, and resume normal Prumo synchronization when available. Aubrieta architecture never depends on a harness service being online.

# Security/privacy

Microcontexts and generated documentation must not contain credentials, signing secrets, private keys, authentication tokens or user artwork/content that is unnecessary for the task. Use synthetic/minimized fixtures. Prumo receives only the repository/spec context required for development.

# Documentation language

Prumo-generated canonical repository documentation is authored in **English (`en-US`) first**. Stable/release changes include the **pt-BR translation in the same completion scope**. Identifiers/code/schema values remain invariant.

# CI/Definition of Done

For Prumo-organized work, completion still requires normal Aubrieta evidence:

- architecture dependency checks;
- tests/gauntlets;
- headless path where applicable;
- module detach proof where applicable;
- UI semantic/token/a11y checks where visible;
- Plugin/MCP conformance when exposed;
- documentation impact manifest complete;
- EN + pt-BR current;
- generated reference current;
- VitePress build/link/example gates passing;
- dossier/handoff closed with honest residual risk.

# Code-agent rule

**Use Prumo to make context smaller and implementation traceable, not to outsource product decisions.** If Prumo output proposes behavior that conflicts with an accepted Aubrieta contract, the agent follows Aubrieta, records the conflict and updates the task material accordingly.