# 14 — Implementation Evidence, Performance, Security & AgentOps

# Evidence hierarchy

Claim < test log < deterministic fixture < sanitizer/fuzzer result < benchmark profile < release artifact verification.

# Test architecture

- C++ unit/property tests;
- Python strict typing/unit tests;
- binding contract tests;
- document roundtrip/migration;
- geometry differential tests;
- color oracle tests;
- raster golden tiles;
- renderer visual goldens;
- UI semantic/interaction tests;
- plugin sandbox tests;
- MCP workflow replay;
- hostile file fuzzing;
- release packaging smoke.

# Performance

Budgets são definidos por hardware tier. Capturar startup, open/save, zoom/pan, transform latency, brush p50/p99, text layout, render frame time, export throughput, RAM/VRAM, long-session growth.

# Security

Threat model cobre .PTND/ZIP bombs, codecs, fonts, SVG/PDF parsers, plugins, MCP, filesystem grants, network brokers, temp files, crash bundles e update supply chain.

# AgentOps

Cada Goal mantém dossier com microcontext, plan DAG, owner agent, skills, commands, evidence paths, known limitations e handoff. Implementer != verifier para gates críticos.

[14.1 — Test Architecture, Fixtures, Golden Projects & Regression Policy](14%201%20%E2%80%94%20Test%20Architecture,%20Fixtures,%20Golden%20Project%203f19bb7d023f81f98f4cd91165db0753.md)

[14.2 — Performance Contract, Hardware Tiers, Benchmarks & Long-Session Budgets](14%202%20%E2%80%94%20Performance%20Contract,%20Hardware%20Tiers,%20Bench%203f19bb7d023f8173ab1ed49787c1b975.md)

[14.3 — Security Verification, Fuzzing, Hostile Inputs & Sandbox Evidence](14%203%20%E2%80%94%20Security%20Verification,%20Fuzzing,%20Hostile%20Inp%203f19bb7d023f81f9a731c7e3a92c3ffd.md)

[14.4 — UI Visual, Semantic, Accessibility & Affinity Workflow Evidence](14%204%20%E2%80%94%20UI%20Visual,%20Semantic,%20Accessibility%20&%20Affini%203f19bb7d023f8167801dd2f4c4332770.md)

[14.5 — .PTND Save/Recovery Torture Suite & Migration Evidence](14%205%20%E2%80%94%20PTND%20Save%20Recovery%20Torture%20Suite%20&%20Migratio%203f19bb7d023f81979404fddedc59e1a7.md)

[14.6 — Release Evidence, Artifact Verification, Rollback & Support Diagnostics](14%206%20%E2%80%94%20Release%20Evidence,%20Artifact%20Verification,%20Ro%203f19bb7d023f81ecb694d9bdb3f2b676.md)

[14.7 — UI/UX Evidence Integration, Semantic Goldens & Affinity Workflow Comparisons](14%207%20%E2%80%94%20UI%20UX%20Evidence%20Integration,%20Semantic%20Golden%203f19bb7d023f815dbcadd1bcfa1ceb5a.md)

[14.8 — AgentOps, Tooling Commands, Evidence Bundles & Deterministic Handoff](14%208%20%E2%80%94%20AgentOps,%20Tooling%20Commands,%20Evidence%20Bundle%203f19bb7d023f815d95cdcc6530f7e3b9.md)