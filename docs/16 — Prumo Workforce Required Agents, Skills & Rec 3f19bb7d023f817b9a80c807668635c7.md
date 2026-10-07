# 16 — Prumo Workforce: Required Agents, Skills & Recipes

# Authority

Esta página define o subconjunto do workforce do Prumo que deve ser copiado para o projeto e usado por Goals. Fonte: [Prumo workforce](https://github.com/poppy-team/prumo/tree/main/src/prumo/resources/workforce).

# Required agents

| Agent | Uso obrigatório |
| --- | --- |
| architect | boundaries, ADR, Plan DAG, contracts |
| systems-architect | memory, concurrency, ABI, cache, native engine |
| editor-engineer | tools, commands, inspectors, undo/save workflows |
| renderer-engineer | render graph, GPU, shaders, visual correctness |
| ui-component-engineer | Qt/PySide component contracts e reusable controls |
| ux-architect | IA, flows, progressive disclosure, workflows |
| design-system-engineer | tokens, component states, density, themes |
| accessibility-reviewer | keyboard, screen reader, contrast, cognitive load |
| performance-agent | profiling, SIMD/cache/GPU evidence |
| security-architect | trust boundaries, permissions, threat model |
| security-reviewer | independent security verification |
| quality-reviewer | clean code, coupling, maintainability |
| tester | deterministic verification suites |
| documentation-maintainer | living canonical docs |
| technology-decision-agent | renderer/libs/ABI choices via Pareto + benchmark |
| release-verifier | artifacts, checksums, rollback, release gates |
| implementer | general implementation under locked contracts |
| reviewer | independent code review |
| debugger | reproduction-first defect isolation |
| explorer | repository/context mapping before large changes |

# Required skills

**Core:** lang-python, lang-cpp, architecture-quality, clean-code, code-quality, code-review, refactoring, concurrency-quality, memory-management, memory-safety-sanitizers, performance-native, benchmarking.

**Graphics:** rendering-2d, shaders, scene-graph, svg-engineering, asset-pipeline, color-science, typography-system.

**Editor/UI:** editor-tooling, input-handling, design-system, design-tokens, component-specification, interaction-design, ux-architecture, user-flows, ui-implementation, ui-ux-review, visual-qa, visual-regression.

**Accessibility:** accessibility, keyboard-accessibility, screen-reader, focus-management, contrast, zoom-reflow, motion-accessibility.

**Extensibility/security:** plugin-architecture, plugin-security, mcp-integration, mcp-tooling, mcp-security, desktop-security, untrusted-project-security, filesystem-security, secure-coding, threat-modeling, supply-chain-security.

**Data/release/docs:** serialization, testing-quality, fuzz-grammar-testing, release-engineering, ci-cd, documentation, documentation-for-llms, documentation-publishing, project-documentation-architect, grounded-implementation, implementation-reality-verification.

**Orchestration:** goal-management, orchestration-multi-agent, prumo-navigation, lean-progressive-context, context-optimization.

# Recipes

Usar conforme mudança: project-bootstrap, architecture-change, feature-standard, engine-renderer, ui-feature, ui-review, design-system-foundation, bug-fix, documentation-refactor, security-review, security-audit-release, release, github-issue.

# Copy policy

O projeto deve versionar cópia/pin do workforce necessário em diretório de tooling/documentado, com origem/commit do Prumo registrados. Não copiar cegamente todo o catálogo se não será usado; copiar o conjunto obrigatório acima e dependências transitivas de skills/recipes.

# Role separation

Architecture, implementation, security review, performance verification e release verification não devem ser colapsados no mesmo agente quando o risco exige independência.

[16.1 — Required Prumo Agents, Responsibilities & Handoff Graph](16%201%20%E2%80%94%20Required%20Prumo%20Agents,%20Responsibilities%20&%20H%203f19bb7d023f813e9326f9cb4b824e25.md)

[16.2 — Required Prumo Skills by Petunia Subsystem](16%202%20%E2%80%94%20Required%20Prumo%20Skills%20by%20Petunia%20Subsystem%203f19bb7d023f81a49f17ff7ac60798e8.md)

[16.3 — Required Prumo Recipes & When to Invoke Them](16%203%20%E2%80%94%20Required%20Prumo%20Recipes%20&%20When%20to%20Invoke%20The%203f19bb7d023f8119bfd8f59f9120ec61.md)

[16.4 — Workforce Vendoring, Pinning, Updates & Project Directory Contract](16%204%20%E2%80%94%20Workforce%20Vendoring,%20Pinning,%20Updates%20&%20Pro%203f19bb7d023f817a8265d54b55d989d6.md)

[16.5 — Task Router: Subsystem → Primary Agent → Mandatory Skills → Reviewer](16%205%20%E2%80%94%20Task%20Router%20Subsystem%20%E2%86%92%20Primary%20Agent%20%E2%86%92%20Man%203f19bb7d023f8136aad0db1c77dc5f12.md)

[16.6 — Agent Handoff Contract, Evidence Payload & Independent Verification Rules](16%206%20%E2%80%94%20Agent%20Handoff%20Contract,%20Evidence%20Payload%20&%20%203f19bb7d023f814ba284e88c740f534f.md)

[16.7 — Workforce Dependency Resolution, Mandatory Skill Closure & Validation](16%207%20%E2%80%94%20Workforce%20Dependency%20Resolution,%20Mandatory%20%203f19bb7d023f817f9264f703d6dbd501.md)

[16.8 — Petunia-Specific Agent Overlays, Domain Briefs & Microcontext Packages](16%208%20%E2%80%94%20Petunia-Specific%20Agent%20Overlays,%20Domain%20Bri%203f19bb7d023f8188bf68d300ab2cca82.md)