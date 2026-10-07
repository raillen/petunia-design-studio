# 30.6 — Registry Generation, Static Validation, Documentation & Deprecation Ledger

# Source

Built-in registry descriptors live in structured code/schema files suitable for generation/introspection; no scattered magic strings.

# Generated artifacts

ID constants/enums where helpful, Python stubs, docs tables, command palette index seeds, MCP discovery schemas, conformance manifests and translation-key checks.

# Static validation

Unique IDs; valid namespace; referenced TextId/IconId/HelpId exists; Action shortcut conflicts checked; Property type/control compatible; capability refs known; deprecation replacement valid.

# Deprecation ledger

ID, deprecated version, replacement, migration behavior, removal target and compatibility tests. Removed plugin IDs remain recognizable in workspace/shortcuts where preservation matters.

# Runtime

Registries can accept plugin contributions through validated namespace/capability permissions; built-in registry immutable after bootstrap except documented dynamic providers.

# CI

Registry diff is reviewed as public semantic API change, not incidental UI edit.