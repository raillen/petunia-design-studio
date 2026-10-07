# 22.4 — Library Search, Tags, Favorites, Sync/Backup, Missing Providers & Performance

# Index

Library service builds searchable index over name, tags, kind, author/source and recently used. Search returns paginated stable IDs.

# Tags

User tags separate from provider built-in categories where practical so updates do not erase organization.

# Favorites/recents

Stored user metadata keyed ResourcePresetId. Provider removal preserves tombstone metadata optionally.

# Missing provider

Plugin library unavailable -> entries show unavailable placeholders if referenced in workspace/doc; document embedded resources remain usable.

# Backup

User libraries live in versioned app data directory and can export archive. Backup/import never includes secrets or plugin permission grants.

# Sync

Cloud/team sync is future provider implementing same library API; conflict semantics must be explicit, no hidden last-write-wins for binary assets.

# Performance

Thumbnail generation async/cached, grid virtualized, 100k entries searchable without widget explosion.

# Tests

Large library, corrupt entry, provider timeout, thumbnail cancellation and backup/restore semantic IDs.