# G02 — Vendor & Pin Prumo Workforce

# Goal

Make Petunia agent orchestration reproducible from a pinned Prumo workforce snapshot.

# Depends

G01.

# Authority

16.1–16.4, 12.10.

# Owner

documentation-maintainer + architect.

# Deliverables

tooling/prumo/[UPSTREAM.md](http://UPSTREAM.md), commit.txt, selected agents/skills/recipes, Petunia overlays, validation script and task router.

# Acceptance

Pin records upstream SHA; required agent manifests and transitive skills exist; CI detects missing/modified vendor files; local overrides are separate.

# Security/licensing

Preserve upstream licensing and do not execute unreviewed mutable main content during deterministic Goal.

# Evidence

Vendor diff, validation output and mapping from Petunia subsystem to agent/skills.