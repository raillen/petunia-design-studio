# 22.1 — Resource Library Model, Scope, Stable IDs, Provenance & Versioning

# Library

ResourceLibrary has LibraryId, kind, scope, display metadata, schema version, provider and entries. Entry has stable ResourcePresetId independent from filename/name.

# Scopes

BuiltIn read-only; User writable; Document embedded; External file/library; PluginProvided dynamic; future Team remote.

# Provenance

Entry records provider/library, created/modified version, optional source license/author and content fingerprint.

# Names

Human names are mutable metadata, never identity. Duplicate names allowed with disambiguation.

# Versioning

Each resource kind has schemaVersion and migration. Library container version independent. Unknown future entry preserved or skipped with issue according required semantics.

# References

Document can embed copy or reference external library entry + fingerprint. Portable workflows should offer Embed/Collect.

# Conflicts

Import collision by stable ID: replace if same provenance/version policy, duplicate with new ID, or keep both. UI never silently overwrite user preset.

# Tests

Rename, duplicate names, plugin disappears, library migration, external ref changed and embedded portability.