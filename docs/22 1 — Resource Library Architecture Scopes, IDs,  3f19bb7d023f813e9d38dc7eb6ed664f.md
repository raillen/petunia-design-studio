# 22.1 — Resource Library Architecture: Scopes, IDs, Providers, Versioning & Provenance

# Resource model

LibraryResourceId stable within provider/library. Document imports receive Document ResourceId while retaining SourceProvenance linking provider/resource/version/fingerprint.

# Scopes

BuiltIn, UserLibrary, Document, Workspace/Profile, PluginProvided, ExternalLinked.

# Provider

IResourceLibraryProvider lists metadata, resolves content, thumbnails, search and optional mutation capabilities. Read-only providers advertise inability to edit/delete.

# Resource metadata

id, type, schemaVersion, name, tags, category, author/provider, license/source optional, created/modified, content fingerprint, preview ref, dependencies, compatibility range.

# Versioning

Resource schema migrates per type. Provider resource update does not silently alter existing document imported copies unless content is linked by explicit relationship.

# Conflicts

Importing same stable ID with different fingerprint invokes conflict policy: Keep Existing, Replace/Update, Duplicate New ID. Never merge by name alone.

# Search

Central ResourceIndex can aggregate providers by type/tags/text without loading full payload.

# Missing provider

Document-owned copies remain. Linked references show unavailable and preserve provenance.

# Backup/sync

User libraries stored in portable folder/database schema; cloud sync is optional provider feature, not core assumption.

# Security

Resource packs treated untrusted; no executable content unless routed plugin system.

# Tests

Provider unavailable, version migration, same-ID conflict, search 100k items and document portability.