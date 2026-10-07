# 07 — Quality, Plugins, MCP & AI Development

# Quality architecture

Quality gates are part of architecture. Python and C++ each have static, dynamic, memory, concurrency, serialization, UI and performance verification.

# Plugin model

Three trust tiers:

1. **Built-in trusted Python extensions** — in-process, shipped with app, reviewed with same standards;
2. **Third-party Python plugins** — out-of-process plugin host with RPC/capability broker and OS sandbox where available;
3. **WASM components** — optional stronger isolation tier using the same semantic SDK.

Native C++ plugins are internal/advanced only unless a future ABI/signing ADR explicitly approves public distribution.

# Plugin permissions

[document.read/write](http://document.read/write), selection, ui.panel, register.tool, import/export, data-source, scoped filesystem, allowlisted network, clipboard, background jobs. Default deny; updates that request more authority require renewed consent.

# MCP

MCP é adapter sobre ActionRegistry, Query API, Job system e semantic inspection. Deve ter discovery, document summaries, object queries, mutations com expected_revision, dry-run/explain, long jobs, export, plugin discovery e UI semantic inspection.

# AI/code-agent rule

Agentes não inventam arquitetura. Todo trabalho começa com Goal + microcontext + contracts + acceptance criteria. Mudança de boundary exige ADR. Implementer não aprova sua própria evidência crítica.

# Security

Arquivos, plugins, codecs, fonts e importers são hostile-input boundaries. Fuzzing, decompression limits, path traversal rejection, sandboxing, no ambient network e no arbitrary process execution.

# Testing layers

- C++ unit/property/golden;
- Python unit/type/contract;
- binding parity;
- save/load roundtrip;
- import/export fixtures;
- renderer visual regression;
- performance benchmarks;
- UI semantic/interaction;
- plugin permission/isolation;
- MCP deterministic workflows;
- long-session torture tests.

# Evidence

Cada milestone produz evidence bundle: revision, environment, commands, test results, sanitizer status, benchmarks, screenshots/semantic goldens, known limitations e release verdict.

[07.1 — Quality Gates, Definition of Done & Severity Taxonomy](07%201%20%E2%80%94%20Quality%20Gates,%20Definition%20of%20Done%20&%20Severit%203f19bb7d023f81778b1bfb1c98ac6bda.md)

[07.2 — Security Threat Model: Documents, Plugins, MCP, Codecs, Filesystem & Supply Chain](07%202%20%E2%80%94%20Security%20Threat%20Model%20Documents,%20Plugins,%20M%203f19bb7d023f81c79f02dcdd430b1bd5.md)

[07.3 — AI / Code-Agent Development Policy, Grounding, Authority & Non-Hallucination Rules](07%203%20%E2%80%94%20AI%20Code-Agent%20Development%20Policy,%20Grounding%203f19bb7d023f8161842dea1b88e180b5.md)

[07.4 — Plugin & MCP Product Governance, Permission Changes, Deprecation & Compatibility](07%204%20%E2%80%94%20Plugin%20&%20MCP%20Product%20Governance,%20Permission%203f19bb7d023f817aaeb2ee342702c98d.md)

[07.5 — Privacy, Telemetry, Diagnostics & Data Handling Policy](07%205%20%E2%80%94%20Privacy,%20Telemetry,%20Diagnostics%20&%20Data%20Hand%203f19bb7d023f811fb7aae3c2995a78d3.md)

[07.6 — Dependency, License, SBOM & Third-Party Intake Policy](07%206%20%E2%80%94%20Dependency,%20License,%20SBOM%20&%20Third-Party%20Int%203f19bb7d023f81218524c6f1e82d7d18.md)

[07.1 — Quality Gates by Change Type & Required Review Roles](07%201%20%E2%80%94%20Quality%20Gates%20by%20Change%20Type%20&%20Required%20Rev%203f19bb7d023f81708c59ca18791df0d3.md)

[07.2 — Threat Model: Documents, Codecs, Fonts, Plugins, MCP, Filesystem & Updates](07%202%20%E2%80%94%20Threat%20Model%20Documents,%20Codecs,%20Fonts,%20Plug%203f19bb7d023f81869816f73b93d6a200.md)

[07.3 — Determinism, Reproducibility & Golden Evidence Policy](07%203%20%E2%80%94%20Determinism,%20Reproducibility%20&%20Golden%20Evide%203f19bb7d023f81fe91baf56ab3d850b1.md)

[07.4 — AI-Assisted Development Boundaries, Agent Safety & Implementation Reality](07%204%20%E2%80%94%20AI-Assisted%20Development%20Boundaries,%20Agent%20S%203f19bb7d023f8101b30bd2ec28ee4dad.md)

[07.5 — Plugin/MCP Semantic Parity & No-Backdoor Rule](07%205%20%E2%80%94%20Plugin%20MCP%20Semantic%20Parity%20&%20No-Backdoor%20Ru%203f19bb7d023f81f6aaf3e689a63676cc.md)

[07.6 — Dependency Risk, Licensing, Supply Chain & Update Governance](07%206%20%E2%80%94%20Dependency%20Risk,%20Licensing,%20Supply%20Chain%20&%20%203f19bb7d023f81ecba18c433763cf0a0.md)