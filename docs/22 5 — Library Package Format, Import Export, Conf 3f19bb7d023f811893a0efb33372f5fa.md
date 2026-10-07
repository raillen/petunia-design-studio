# 22.5 — Library Package Format, Import/Export, Conflict Resolution, Backup & Sync Boundaries

# Package

A Petunia library package is a bounded archive with manifest, resource entries, previews and dependency index. Exact extension/media type gets ADR before public release.

# Manifest

libraryId, schemaVersion, libraryKinds, title/author/version, resources[{id,kind,schemaVersion,path,hash}], dependencies, license/provenance and required capabilities.

# Security

Same safe-archive rules as PTND: no traversal, symlinks, bombs, duplicate canonical paths or unbounded decode.

# Import conflict matrix

Same ID + same hash = reuse.

Same ID + different hash = Replace, Import Copy(New ID) or Cancel.

Different ID + same content = keep semantic IDs unless dedup policy explicitly safe.

Missing dependency = import degraded/unavailable with report, not silent substitution.

# Backup

User library directory/package can be exported atomically with manifest. Automatic backup policy separate from cloud sync.

# Sync boundary

No cloud provider is canonical. Future sync adapters exchange versioned library packages/operations and must solve conflict/identity explicitly.

# Tests

Corrupt package, conflict paths, missing dependency, read-only destination, backup restore and schema migration.