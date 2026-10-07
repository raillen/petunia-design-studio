# 16.1 — Required Prumo Agents, Responsibilities & Handoff Graph

# Source

All names below correspond to directories under src/prumo/resources/workforce/agents in poppy-team/prumo.

# Core decision roles

- **architect** — owns boundaries, ADRs, Plan DAG and interface contracts.
- **technology-decision-agent** — candidate elimination/Pareto/benchmark plan for renderer, libs, runtimes.
- **systems-architect** — native memory/concurrency/cache/ABI.
- **ux-architect** — IA, task flows, progressive disclosure.
- **design-system-engineer** — token/component contract.

# Implementation roles

- **implementer** — generic scoped implementation.
- **editor-engineer** — tools, commands, inspectors, assets, save/undo workflows.
- **engine-engineer** — engine/runtime subsystems when scope crosses lower-level services.
- **renderer-engineer** — render pipeline/GPU/shaders.
- **ui-component-engineer** — reusable accessible UI controls.
- **svg-artist** — Petunia icon/vector asset production when needed.
- **documentation-maintainer** — living docs after verified behavior.

# Investigation

- **explorer** — map repository/entry points before changes.
- **debugger** — reproduce/isolate defects before patch.
- **design-researcher** — evidence around interaction/reference workflows.
- **prototyper** — disposable UI/technology spikes.

# Independent verification

- **tester** — deterministic functional test code/evidence.
- **reviewer** — code review.
- **quality-reviewer** — maintainability/architecture quality.
- **performance-agent** — profiler/counters/benchmarks.
- **accessibility-reviewer** — WCAG/keyboard/screen reader/cognitive.
- **security-architect** — threat model/policy.
- **security-reviewer** — independent audit.
- **release-verifier** — artifact/release/rollback.

# Optional/specialized

devops-engineer for CI/release infrastructure; visual-identity-auditor for design identity drift; creative-director for large visual direction changes; isolation-auditor for plugin/process sandbox.

# Canonical handoff examples

Feature tool:

explorer -> architect/editor-engineer -> implementer/editor-engineer -> tester -> reviewer + quality-reviewer -> accessibility-reviewer if UI -> documentation-maintainer.

Renderer:

technology-decision-agent -> architect -> renderer-engineer -> tester -> performance-agent -> reviewer -> release-verifier.

Plugin/MCP:

security-architect + architect -> implementer -> tester -> security-reviewer -> documentation-maintainer.

# Independence

Security reviewer, performance verifier and release verifier must not simply self-approve implementation when their Prumo role requires independence.