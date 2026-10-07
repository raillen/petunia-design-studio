# 09.18 — Repository Topology, Clean Architecture, Python Typing, C++ Safety & ADR Governance

# Suggested repository

```
/apps/petunia_studio
/python/petunia_app
/cpp/petunia_core
/cpp/petunia_geometry
/cpp/petunia_raster
/cpp/petunia_text
/cpp/petunia_color
/cpp/petunia_render
/cpp/petunia_io
/cpp/petunia_jobs
/bindings/python
/shaders
/resources
/schemas
/plugins/sdk
/mcp
/tests
/benchmarks
/fixtures
/docs
/tooling/prumo
```

# Python architecture

domain-facing Protocols and immutable DTOs; services by capability; UI widgets thin. Strict pyright forbids implicit Any on product code. Type stubs for native extension generated/maintained as contract.

# C++ architecture

Value types + domain services + explicit ownership. RAII, no owning raw pointers, no global mutable services, no QObject in core. Includes obey layer direction.

# Clean code budgets

Functions/classes cohesive; avoid Manager/Helper god types; feature modules have clear public facade; dependency injection through constructors/factories; no speculative abstraction before multiple uses or real boundary.

# ADR triggers

Renderer/backend selection, canonical pixel format, public plugin ABI, format breaking change, concurrency model replacement, text/color engine replacement, security trust-boundary change.

# Review

Architect owns boundary ADR; technology-decision-agent supplies evidence for tech choices; implementation independently reviewed; security/performance gates by appropriate agents.

# Generated code

Bindings/stubs/schema code clearly separated; generated output never hand-edited; source schema/tool version pinned.