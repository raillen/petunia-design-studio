# 14.8 — AgentOps, cargo xtask, CI Commands, Evidence Bundles & Deterministic Handoff

<aside>
🤖

Humans, Prumo and any code agent need one deterministic way to ask “is this change actually ready?”

</aside>

# Canonical command facade

Use a repository-owned task runner, preferably cargo xtask, as the stable developer/CI facade. Exact implementation may evolve, but semantic commands should converge on:

- verify;
- architecture;
- test;
- conformance;
- fixtures;
- fuzz-smoke;
- bench-smoke;
- ui-gauntlet;
- security;
- migrations;
- docs;
- release-check;
- gauntlet.

Names may be finalized during repository implementation, but scripts/agents must not duplicate these workflows independently.

# verify

Fast default for local/PR use: formatting, lint, compile, unit/property/contract tests, dependency-boundary checks and documentation consistency appropriate to the changed scope.

# gauntlet

Reads the work/impact manifest and runs all applicable evidence classes rather than one hard-coded universal mega-suite.

# Evidence manifest

Every substantial Wave emits a machine-readable manifest with Goal/Wave IDs, changed FeatureIds, relevant Fixture/Test/Fuzz/Benchmark/UI IDs, commands run, results, environment/build metadata, docs status and residual risks.

# Changed-scope selection

CI determines relevant suites from semantic manifests/registries plus changed code, not filename guesses alone. A full scheduled run catches selection mistakes.

# Reproducibility

Commands print versions, feature flags, seed, target platform and corpus revision. Failed cases output a single reproduction command or fixture reference where practical.

# Prumo

Prumo may create/update dossiers and orchestrate these commands, but Aubrieta repository commands remain usable without Prumo.

# Handoff

An agent handoff links exact evidence rather than saying “tests pass”. It states what was not run and why.

# CI lanes

PR: fast deterministic gate.

Merge/main: broader integration/conformance.

Nightly: long fuzz, large corpus, full benchmarks, soak.

Release: packaging/platform/signing/update + migration/security/UI evidence bundle.

# Failure policy

No automatic baseline acceptance. A changed golden, benchmark threshold or compatibility result requires review against the canonical specification.

# Documentation

Generated evidence indexes can be published in VitePress, but raw logs need not become user docs. Public docs describe guarantees; evidence pages link reproducible artifacts where appropriate.