# 12 — Code Agent Implementation Handbook & Living Documentation

# Amendment 2026-09-21 — Total Assurance Documentation

O handbook passa a consumir a **Total Assurance Constitution do Prumo** como metacontrato de qualidade. Documentation readiness exige não apenas descrição suficiente, mas Surface Coverage, Acceptance→Evidence traceability, negative guarantees, failure/recovery, NFR/performance budgets, security boundaries, oracle quality, freshness e proof-of-use.

A implementação Slint não pode ser considerada completa por screenshots ou compilação.

<aside>
📚

**Purpose:** this handbook defines how humans and code agents turn the Aubrieta notebook into implementation without inventing hidden architecture, UI conventions or undocumented APIs. Documentation is part of the product and part of Definition of Done.

</aside>

# Operating model

Aubrieta uses the notebook as the canonical **decision/specification layer** and repository documentation as the canonical **implementation-facing, version-pinned publication layer**. The public/live documentation site is built from repository Markdown with VitePress.

No implementation is considered complete while these layers disagree.

# Source-of-truth hierarchy

When sources conflict, use this order unless an explicit superseding ADR says otherwise:

1. accepted/recent ADR or explicitly resolved decision;
2. canonical Aubrieta root + relevant Architecture/Functional/Interface Atlas page;
3. implementation contract in repository docs for the current branch/version;
4. tests/fixtures that encode the accepted contract;
5. code behavior;
6. historical notebooks/examples.

If code contradicts a newer accepted specification, do not silently redefine the documentation to match the bug.

# Documentation languages

- **English (`en-US`) is the canonical source language for repository/VitePress documentation.**
- **Brazilian Portuguese (`pt-BR`) is the mandatory first translation.**
- Identifiers, API method names, code symbols, file formats and schemas remain invariant and are not translated.
- A translation may clarify prose but cannot change normative meaning.
- English source updates invalidate/stale the corresponding pt-BR translation until synchronized.

# Documentation categories

Every stable subsystem maps into one or more categories:

- Getting Started;
- User Guide;
- Concepts;
- UI/UX & workflows;
- Architecture;
- Implementation Guides;
- Core API/reference;
- Plugin SDK;
- MCP/Automation API;
- File formats/interoperability;
- Security/permissions;
- Testing/Gauntlets;
- Contributing/Code Agent protocol;
- ADRs;
- Troubleshooting/Diagnostics;
- Glossary.

# Code-agent contract

Before implementation an agent must:

1. read the root charter and relevant Atlas index;
2. fetch the exact canonical subsystem pages;
3. assemble a small implementation micro-context;
4. identify invariants, dependencies, public contracts, tokens/resources and security boundaries;
5. define acceptance tests and documentation impact;
6. detect ambiguity/contradiction before coding.

After implementation it must:

1. run the subsystem gauntlet;
2. run architecture/modularity checks;
3. update English documentation;
4. update pt-BR translation;
5. build/test VitePress;
6. update ADR/Atlas pages when behavior or contract changed;
7. report residual risks honestly.

# No undocumented invention

A code agent may choose reversible local implementation details when the contract leaves freedom. It may **not** silently choose a new persisted schema, public API, plugin permission, interaction pattern, user-facing term, design token, cross-module dependency or irreversible behavior. Those require specification/ADR update.

# Documentation is executable where possible

Prefer documentation generated or checked from source metadata:

- Action/Property/Capability registries;
- TextId/IconId/token catalogs;
- plugin/MCP schemas;
- diagnostics codes;
- format schemas;
- command examples;
- tested code snippets.

Manual prose explains intent and usage; machine-readable truth should not be copied by hand into multiple conflicting tables.

# VitePress publication

Repository documentation is rendered as a multilingual VitePress site, published automatically after CI passes. The site must remain usable offline after build where practical and must not depend on a proprietary backend for basic reference.

# Required child guides

This handbook is expanded by dedicated pages for source governance, agent workflow, VitePress/i18n, implementation planning/DoD, API documentation, quality gauntlets and repository contribution rules.

# Global rule for agents

**Do not optimize for the smallest patch if the patch violates the architecture. Optimize for the smallest complete change that preserves Aubrieta's semantic contracts, modularity, portability, safety, usability and documentation integrity.**

[12.1 — Source of Truth, Canonical Documentation, Language Policy & Drift Resolution](12%201%20%E2%80%94%20Source%20of%20Truth,%20Canonical%20Documentation,%20L%203db9bb7d023f81b08c9fd60fe7dc97bb.md)

[12.2 — Code Agent Operating Protocol, Microcontexts, Gauntlet Loop & Decision Discipline](12%202%20%E2%80%94%20Code%20Agent%20Operating%20Protocol,%20Microcontext%203db9bb7d023f81eba360d7f459ebac35.md)

[12.3 — Living Documentation Pipeline: VitePress, i18n, CI/CD, Versioning & Site Information Architecture](12%203%20%E2%80%94%20Living%20Documentation%20Pipeline%20VitePress,%20i1%203db9bb7d023f8168a7b6e4d378b02d12.md)

[12.4 — Implementation Plan Template, Definition of Ready, Definition of Done & ADR Triggers](12%204%20%E2%80%94%20Implementation%20Plan%20Template,%20Definition%20of%203db9bb7d023f819687aae98591d81fcb.md)

[12.5 — API Documentation Standard for Core, Plugin SDK, MCP, Schemas & Diagnostics](12%205%20%E2%80%94%20API%20Documentation%20Standard%20for%20Core,%20Plugin%203db9bb7d023f81ae9b64f4236b9f7a03.md)

[12.6 — Documentation Quality Gauntlet, Translation Freshness, Accessibility & Release Gates](12%206%20%E2%80%94%20Documentation%20Quality%20Gauntlet,%20Translation%203db9bb7d023f816f97acdec5a8a82f79.md)

[12.7 — Repository, Contribution, Review & Documentation Maintenance Rules for Agents](12%207%20%E2%80%94%20Repository,%20Contribution,%20Review%20&%20Document%203db9bb7d023f8166a88fdc1a5fa53e03.md)

[12.8 — Requirement Status, Scope Taxonomy, Backlog Semantics & Ambiguity Elimination](12%208%20%E2%80%94%20Requirement%20Status,%20Scope%20Taxonomy,%20Backlog%203db9bb7d023f81a0b529c767560d3f7f.md)

[12.9 — Change Impact Matrix: What Agents Must Update for Every Kind of Feature Change](12%209%20%E2%80%94%20Change%20Impact%20Matrix%20What%20Agents%20Must%20Updat%203db9bb7d023f81ceaf86dbf91a4bd2bf.md)

# Prumo CLI requirement

Substantial implementation/documentation work must use the **installed Prumo CLI as the project documentation/harness workflow**, according to the dedicated 12.10 guide. Prumo is an orchestration aid, not an Aubrieta source dependency or alternate source of product truth. Agents must inspect the installed CLI/version/help rather than inventing unsupported command syntax.

[13 — Full Notebook Page-by-Page Audit & Conformance Ledger](13%20%E2%80%94%20Full%20Notebook%20Page-by-Page%20Audit%20&%20Conformanc%203db9bb7d023f810fb6fdf8aa804ef74e.md) records notebook-wide audit findings and residual documentation risk. Stable code-agent work should treat unresolved High/Critical ledger items as implementation blockers when they touch the task.

[12.10 — Prumo CLI Workflow, Goals/Waves, Implementation Dossiers, Microcontexts & Agent Handoffs](12%2010%20%E2%80%94%20Prumo%20CLI%20Workflow,%20Goals%20Waves,%20Implement%203dc9bb7d023f81ba8bbfd27c9cb81baf.md)