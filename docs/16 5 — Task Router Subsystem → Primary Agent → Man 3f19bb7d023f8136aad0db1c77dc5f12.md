# 16.5 — Task Router: Subsystem → Primary Agent → Mandatory Skills → Reviewer

# Purpose

Make agent selection deterministic for common Petunia work.

# Routing examples

Document/Commands -> editor-engineer + architect; skills lang-cpp, editor-tooling, architecture-quality; review quality-reviewer/tester.

Geometry -> systems-architect/editor-engineer; lang-cpp, performance-native, benchmarking; review tester/performance-agent.

Raster/Brush -> engine-engineer; rendering-2d, concurrency, memory-management; review performance-agent/tester.

Renderer -> renderer-engineer; shaders, rendering-2d, performance-native; review performance-agent/quality.

Qt UI -> ui-component-engineer; lang-python, ui-implementation, design-system; review accessibility-reviewer/quality.

UX flow -> ux-architect; ux-architecture, interaction-design; review accessibility.

PTND/IO -> editor-engineer + security-architect for hostile parsers; serialization, filesystem-security, fuzzing.

Plugin/MCP -> architect + security-architect; plugin/mcp skills; independent security-reviewer.

Release -> release-verifier + devops; release-engineering, supply-chain.

# Multiple domains

Choose one primary owner by highest-risk semantic boundary; add specialists as reviewers/Tasks rather than vague shared ownership.

# Escalation

If Goal changes locked architecture, route to architect/technology-decision-agent before implementer proceeds.