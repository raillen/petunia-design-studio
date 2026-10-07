# 12.10 — Prumo CLI Workflow, Goals/Waves, Dossiers, Microcontexts & Agent Handoffs

# Prumo use

Pin the selected workforce and use project-supported Prumo CLI commands/workflows as orchestration layer when available in the vendored version. Exact CLI syntax comes from pinned Prumo docs, not this page.

# Goal

Create Goal with scope, criteria, constraints, authority links and evidence requirements.

# Wave

Group independent Tasks that can run concurrently without editing conflicting ownership/boundaries. Dependencies form Plan DAG.

# Dossier

Store impact map, ADR references, task outputs, test commands, evidence and handoff notes in repository-defined goal directory.

# Microcontext

Generate minimum authoritative context per task: exact docs/headers/schema/tests/nearby implementation. Rebuild when upstream Task changes assumptions.

# Handoff

Agent output records changed files, behavior, commands run, failures/skips, evidence and next role. Reviewer/verifier works from immutable revision when possible.

# Update

When Prumo upstream changes commands/manifest contracts, 16.4 vendoring process updates this workflow and project router.