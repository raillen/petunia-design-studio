# G002 — Vendor & Pin Prumo Workforce

# Goal

Create deterministic local workforce snapshot and project routing policy.

# Depends

G001.

# Primary

documentation-maintainer + architect.

# Skills

prumo-navigation, orchestration-multi-agent, context-optimization, documentation, supply-chain-security.

# Deliverables

tooling/prumo structure from 16.4; [UPSTREAM.md](http://UPSTREAM.md); commit pin; selected agents/skills/recipes; Petunia overrides directory; verification script listing missing transitive skills; task-to-agent router; update procedure.

# Acceptance

A clean checkout can identify exact upstream commit and every required workforce file without network ambiguity. Vendored snapshot differs from upstream only through explicit Petunia overlays outside vendor tree.

# Tests

Manifest path validation; transitive skill references resolve; hash/pin verification; docs links valid.

# Security

Record upstream provenance/license; no executable hook automatically trusted solely because vendored.

# Non-goals

Implementing Goals through Prumo yet; editing upstream workforce semantics.