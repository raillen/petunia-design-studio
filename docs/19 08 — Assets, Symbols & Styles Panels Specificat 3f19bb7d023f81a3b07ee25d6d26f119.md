# 19.08 — Assets, Symbols & Styles Panels Specification

# Assets

PanelId ptnd.panel.assets. Grid/list, categories/tags, search, favorites, source library selector and lazy thumbnails.

# Place

Drag/click/keyboard Place creates semantic asset placement transaction. Missing external dependency gets explicit state.

# Symbols

PanelId ptnd.panel.symbols. Definitions list, instance count, overrides badge, create from selection, edit definition, detach instance and locate instances.

# Styles

PanelId ptnd.panel.styles. Object, character and paragraph styles organized by type/group/tags. Apply, create from selection, update/redefine, duplicate and delete with dependency policy.

# Libraries

All panels consume ResourceLibrary architecture and can show BuiltIn/User/Document/Plugin/External scopes.

# Scale

Virtualized thousands of assets/styles; background thumbnails and search index.

# Tests

Library disconnect, duplicate IDs/conflicts, symbol overrides, style deletion with references and keyboard placement.