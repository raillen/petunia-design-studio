# 09.29 — MCP API Usability, Agent Contracts, Safety, Discovery & Deterministic Automation

# Method documentation

Every MCP tool/resource has stable ID, intent, input/output JSON Schema, permissions, side effects, revision semantics, undo/transaction behavior, examples, error codes and related methods.

# Cookbook

Inspect current document; create/style vector; query/edit by property; add Photo adjustment; batch rename/rearrange layers; Data Merge preview/generate; preflight/export; repair missing resource; inspect plugin capabilities; run semantic UI gauntlet.

# Discovery

Filterable/paginated lists for actions, properties, formats, sessions and plugin contributions. Return compact summaries first.

# Errors

code, category, technical message, field/path, required capability/permission, recoverable flag, deterministic next action/help topic. No opaque invalid-request message.

# Dry run

Operations with destructive conversion, file output, mass mutation or export degradation support explain/dry-run if meaningful.

# File grants

Prefer opaque selected file/directory grants. Remote clients do not get arbitrary absolute path authority just because server runs locally.

# Testing

Cookbook workflows replay on golden projects and compare canonical snapshots with headless UI-equivalent action execution.