# 19.01 — Layers Panel Specification

# Identity

PanelId ptnd.panel.layers. Default right dock, multi-instance false by default, available in Design and Photo.

# Model

QAbstractItemModel over revisioned LayerTreeSnapshot. Rows keyed by ObjectId, never native pointer.

# Row anatomy

disclosure, thumbnail, type icon, name, badges (mask/effect/symbol/link/missing), visibility, lock. Active pixel/mask target gets distinct focus border independent from row selection.

# Selection

Single/multi select synchronized with View SelectionState. Shift range, Ctrl/Cmd toggle. Keyboard arrows navigate hierarchy; Left/Right collapse/expand.

# Drag/drop

Insertion zones distinguish reorder, nest, clip/mask target. Drop preview must make semantic result obvious. Model validates cycles, locked parents, incompatible targets before commit.

# Inline actions

Visibility/lock toggles invoke Actions; modifier can apply recursively only if status/tooltip makes it explicit. F2/double click rename.

# Context menu

Group/Ungroup, Arrange, Duplicate, Delete, Create Mask, Clip, Convert, Rasterize, Symbol, Export and resource-specific actions filtered by capabilities.

# Thumbnails

Derived async; stale thumbnails discarded by object revision. Large trees virtualized; no QWidget per row.

# Search/filter

Filter by name/type/tag/effect/hidden/locked optional. Filtering does not change document hierarchy.

# Persistence

Panel UI state may remember expansion/filter; document stores hierarchy only.

# Accessibility/tests

Tree roles, row action names, active target announced. Test 100k rows, drag nesting, missing plugin object, selection sync, thumbnail churn, keyboard-only hierarchy operations.