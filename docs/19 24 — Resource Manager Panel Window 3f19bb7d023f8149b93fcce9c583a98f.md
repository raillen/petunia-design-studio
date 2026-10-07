# 19.24 — Resource Manager Panel / Window

# Identity

PanelId/window ptnd.resource_manager.

# Rows

ResourceId, type, display name, Linked/Embedded, status OK/Missing/Changed/Corrupt, path/provider/grant summary, size/profile and usage count.

# Actions

Relink, Replace, Embed, Unembed/Link, Reveal, Locate Uses, Refresh, Collect/Package resources, Remove Unused.

# Security

Path/grant details redacted appropriately. Relink through file dialog grant; filename match alone never auto-accepts changed resource.

# Changed resource

Compare fingerprint/metadata, offer Update/Keep Current/Embed Old if available. Update invalidates dependent derived caches and may change layout/appearance with preview/report.

# Missing

Document retains ResourceId and placeholder. Batch relink same folder only after deterministic candidate validation.

# Tests

Missing/moved files, hash changed, permission revoked, embedded conversion, font/image/data resources and undo where mutation is document-semantic.