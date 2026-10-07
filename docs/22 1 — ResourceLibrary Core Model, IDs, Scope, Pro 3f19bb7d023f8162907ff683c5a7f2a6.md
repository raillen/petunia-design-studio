# 22.1 — ResourceLibrary Core Model, IDs, Scope, Provenance & Versioning

# Core

ResourceLibrary is a typed collection descriptor with LibraryId, kind(s), scope, version, provenance, permissions/read-only state, index metadata and migration version.

# Scopes

BuiltIn: shipped immutable resources.

User: editable local resources.

Document: embedded/referenced resources required by PTND.

Plugin: provider-owned contribution, unavailable when plugin disabled.

External: file/cloud/team library through adapter.

# IDs

ResourcePresetId is stable and namespaced. Display name is mutable metadata. Import conflicts compare ID + content fingerprint + kind/version.

# Provenance

Author/source/license/source URL/version/hash/signature metadata where applicable. Provenance affects trust/UI, never semantic rendering unless resource payload explicitly does.

# Versioning

Each resource kind has schemaVersion. Library version is container/index version. Migration is stepwise and preserves unknown optional metadata.

# Dependencies

Preset may reference ColorSwatch, texture ResourceId, font/style, subpreset etc. Dependency graph must reject cycles where semantics cannot support them.

# Search index

Name, tags, category, kind, favorites, recent and provider fields. Index is derived and can rebuild.

# Tests

ID conflicts, unavailable plugin provider, schema migration, dependency missing/cycle, read-only library and search rebuild.