# 19.11 — Assets Panel

# Identity

PanelId ptnd.panel.assets.

# Sources

Built-in/user libraries, document library, plugin/external providers. Source badges show read-only/offline/missing.

# Model

Virtualized grid/list with thumbnail, name, type, tags, provider. Search and categories.

# Placement

Drag to canvas or Place action. AssetInstantiationService creates semantic objects/resources; external content copied/linked according policy.

# Management

Create asset from selection, rename, tag/category, duplicate, delete, import/export library. Read-only providers allow Copy to Library.

# Thumbnail

Async derived rendering with bounded cache.

# Missing provider

Document embedded dependencies remain valid; provider disappearance never corrupts document.

# Accessibility

Grid/list roles, searchable names/tags and Place via keyboard.

# Tests

Large library, plugin source, asset with fonts/images/symbols, missing provider and migration.