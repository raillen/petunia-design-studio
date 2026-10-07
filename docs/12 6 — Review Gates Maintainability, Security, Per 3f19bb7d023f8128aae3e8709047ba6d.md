# 12.6 — Review Gates: Maintainability, Security, Performance, Accessibility & Release Claims

# Maintainability

Quality Reviewer checks cohesion, coupling, dependency direction, naming, complexity, duplication and testability. Metrics inform review but do not replace judgment.

# Security

Security Architect defines threat controls; Security Reviewer audits diff against model, dependency/secret scans and runtime evidence. Reviewer does not fix own findings.

# Performance

Every claimed speedup needs reproducible fixture, profiler before/after and statistical benchmark. “Feels faster” is not evidence.

# Accessibility

New component/tool path must have keyboard route, accessible names/states, focus behavior and contrast. Canvas-specific semantics documented.

# Renderer

Visual changes require deterministic fixture, backend/device profile and tolerance. GPU benchmark evidence for hot path changes.

# File format

Schema/migration change requires old/new fixtures, corruption/load/save tests, minimum reader compatibility and documentation.

# Plugin/MCP

Permission/contract changes require security + semantic parity tests. No expanded authority without user-facing permission explanation.

# Release

Release Verifier checks immutable commit, artifact inventory, signatures/checksums, tests, sanitizer status, migration, docs, licenses/SBOM and rollback.