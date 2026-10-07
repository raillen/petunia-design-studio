# 17.2 — Subsystem Ownership Matrix: Code, Agents, Skills, Tests & Evidence

# Matrix

| Subsystem | Primary agent | Core skills | Mandatory evidence |
| --- | --- | --- | --- |
| Document/Commands | architect + editor-engineer | lang-cpp, architecture-quality, serialization | unit/property/roundtrip |
| Qt shell | ui-component-engineer | lang-python, ui-implementation, accessibility | semantic UI + visual |
| Geometry | engine/editor engineer | lang-cpp, performance-native, benchmarking | golden + fuzz + perf |
| Raster/Brush | engine-engineer | rendering-2d, memory-management, concurrency | pixel goldens + perf |
| Renderer | renderer-engineer | shaders, performance-native, visual-regression | GPU/CPU goldens + traces |
| Text | engine/editor engineer | lang-cpp, typography-related contracts | Unicode corpus + layout perf |
| Color | systems/engine engineer | color-science + native color spec | oracle patches + profiles |
| PTND/IO | editor-engineer | serialization, filesystem-security, fuzzing | roundtrip + torture |
| Plugins | architect + security-architect | plugin-architecture, plugin-security | sandbox/permission tests |
| MCP | architect/implementer | mcp-integration, mcp-security | cookbook parity |
| Design System | design-system-engineer | design-system, tokens, accessibility | component gallery |
| Release | release-verifier | release-engineering, supply-chain | artifact scorecard |

# Review independence

Security, performance and release claims require independent role according risk.

# Ownership is not monopoly

Primary agent owns contract consistency; implementation can be distributed by tasks/waves under same authority.