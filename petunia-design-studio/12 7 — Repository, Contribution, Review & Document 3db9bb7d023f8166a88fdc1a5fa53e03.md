# 12.7 — Repository, Contribution, Review & Documentation Maintenance Rules for Agents

# Repository philosophy

Repository structure should make architectural boundaries obvious enough that a code agent can infer allowed dependencies from location and CI rules.

# Change workflow

1. inspect canonical docs;
2. create implementation microcontext/plan;
3. update/add tests before or with behavior;
4. implement smallest complete cohesive change;
5. run targeted + affected-suite tests;
6. run architecture/security/UI gauntlets;
7. update docs EN then pt-BR;
8. build VitePress/generated reference;
9. summarize compatibility/risk.

# Branch/PR scope

Prefer one coherent architectural feature/fix per PR. Split mechanical repository-wide renames from semantic behavior changes when that improves reviewability.

# Commit messages

Commits should describe intent and subsystem, not agent activity. Avoid `AI changes`, `misc fixes`, or generated-noise commits as final history.

# Reviews

Review checklist asks:

- does dependency direction remain valid;
- can optional module be disabled;
- is business logic duplicated in UI/plugin/MCP;
- are persisted/public contracts versioned;
- are resources tokenized;
- are errors structured;
- are security capabilities minimal;
- are tests meaningful;
- are docs and translations synchronized.

# Generated files

Generated reference/catalog files are reproducible. The generator/source metadata is committed or available in CI. Agents should not manually patch generated output without updating generator/source.

# Fixtures

Keep minimized regression fixtures with provenance and purpose. Hostile/fuzz corpus files avoid real private user data.

# Dependency additions

A new dependency documents reason, ownership/maintenance signal, license, security implications, feature footprint, alternatives considered and which adapter boundary contains it. UI convenience dependencies do not enter domain crates.

# Refactoring rule

Refactor toward documented boundaries; do not rewrite stable code merely for stylistic preference. Large refactors require invariant/behavior tests before migration.

# TODO policy

TODO/FIXME in stable contracts must link to issue/ADR or state precise condition. `TODO later` is not accepted architecture.

# Documentation maintenance

Every PR touching user-visible behavior, public API, persisted data or architecture must include docs impact check. `No docs change` is acceptable only with reason.

# Agent handoff

When an agent stops before full completion, leave a structured handoff: current state, passing/failing tests, files/contracts changed, unresolved decisions, exact next safe step and documentation status. Never leave future agents to infer half-completed migrations from code alone.

# Prumo CLI and repository discipline

Prumo CLI use must leave reviewable repository artifacts, not opaque agent state. Agents record which Prumo workflow/context was used in the implementation dossier/handoff, but final Git history describes Aubrieta intent rather than tool activity.

Never vendor/copy the Prumo implementation into Aubrieta to obtain harness behavior. Aubrieta depends only on its own repository contracts; Prumo is an external installed development tool.

# Handoff artifact minimum

A partial or completed handoff must include:

- goal/work package ID and scope status;
- canonical references read;
- current branch/commit when relevant;
- files/modules changed;
- schema/API/format compatibility effects;
- tests/gauntlets executed with outcomes;
- generated docs/reference status;
- EN and pt-BR status;
- VitePress build status;
- known failures/risks;
- Open ADRs or blocked dependencies;
- exact next safe action.

Do not leave only conversational context or hidden agent scratchpad as the continuation mechanism.