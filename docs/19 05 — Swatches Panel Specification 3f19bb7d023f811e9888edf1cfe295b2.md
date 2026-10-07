# 19.05 — Swatches Panel Specification

# Identity

PanelId ptnd.panel.swatches.

# Sources

Document palette, application/user libraries, imported/plugin palettes. Source selector visible.

# Entries

Process, Global, Spot, Registration, Gradient. Badges distinguish semantic types without color-only encoding.

# Interaction

Click applies to active fill/stroke target. Double-click/edit modifies swatch definition; Global edits propagate. Drag reorders within writable palette.

# Library

Search/tag/favorites for large palettes; create/import/export palette; missing external library handled.

# Context

Create from current color, duplicate, rename, delete, convert global/process/spot where semantically valid.

# Delete

Deleting referenced Global/Spot requires policy: replace consumers with current resolved color, block, or remap; explicit dialog/command.

# Tests

Global propagation, spot reference, palette import conflict, delete referenced swatch and accessibility.