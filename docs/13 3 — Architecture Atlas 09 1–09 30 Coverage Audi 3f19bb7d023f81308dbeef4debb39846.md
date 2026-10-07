# 13.3 — Architecture Atlas 09.1–09.30 Coverage Audit

# Coverage

09.1–09.30 are present in new notebook, preserving architecture subject areas while replacing language/toolkit-specific internals.

# Major replacements

- Cargo/crate topology -> CMake targets + Python packages;
- Rust lifetime/type boundary -> C++ ownership + nanobind/GIL;
- Vello/wgpu fixed page -> backend-neutral RenderScene + renderer decision spike;
- mlua primary plugin runtime -> trusted Python + out-of-process Python + optional WASM;
- GPUI/Slint presentation model -> Qt adapters over UI-agnostic core.

# Preserved invariants

Document model, stable IDs, Commands/transactions, derived caches, geometry, raster, text, color, native format, import/export, jobs, platform services, security, observability, release, preferences, session lifecycle, property schemas, ADR governance, plugins and MCP.

# Audit status

Architecture topic parity: **Mapped/Expanded**.

Open decisions are explicit in 09.26; they are not undocumented gaps.