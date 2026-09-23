# 14.1 — Test Architecture, Conformance Matrix & Regression Policy

<aside>
✅

Every implemented behavior maps to the smallest set of tests that can prove its contract and prevent recurrence of known failures.

</aside>

# Test layers

The canonical order is: unit invariants → property tests → contract tests → integration → headless E2E → UI semantic conformance when visible → plugin/MCP/Lua parity when exposed → fuzz/security → migration/compatibility → performance → documentation examples.

Not every feature needs every layer, but omitted layers require an explicit Not Applicable rationale in its dossier.

# Feature evidence matrix

Each FeatureId records:

- canonical specification/ADR;
- affected crates/modules;
- UnitTestIds;
- PropertyTestIds;
- FixtureIds;
- ConformanceIds;
- FuzzTargets;
- BenchmarkIds;
- UIWorkflowIds;
- MigrationFixtures;
- documentation-example tests.

# Unit tests

Unit tests prove local behavior and errors. They must not mock away the invariant they claim to prove.

# Property tests

Use property testing where state spaces are combinatorial: geometry, transforms, serialization round trips, document mutations, IDs, range/unit parsing, selection set operations, color transforms and migration invariants.

# Contract tests

Ports/adapters, registries, import/export, jobs, Plugin SDK, MCP, platform services and GUI bridges receive contract suites reusable across implementations.

# Headless E2E

Representative user outcomes must work without GPUI when UI is not semantically required. Headless failures indicate business logic leaked into the shell.

# Regression policy

A fixed bug receives a minimized regression test or fixture whenever practical. A manual-only bug fix is incomplete when an automated reproduction is feasible.

# Determinism

Seed random generators. Freeze locale/timezone/theme where relevant. Record tolerance rules explicitly. Do not hide nondeterministic failures behind retries.

# Snapshots and goldens

Structural snapshots use stable semantic data and ignore volatile IDs only when those IDs are proven irrelevant. Pixel goldens define tolerance and rendering environment.

# Test naming

Names describe condition and expected behavior, not implementation function names. Failure output identifies FeatureId/FixtureId and the violated contract.

# CI layers

PR fast gate: formatting/lint, unit/property smoke, contract/integration fast corpus, architecture checks, docs validation.

Scheduled/deep gate: long fuzz campaigns, stress corpus, full benchmark matrix, platform packaging, large migration chains and long-running soak tests.

# Release rule

A feature cannot be labeled Proven for Milestone when its applicable conformance row is missing or stale.