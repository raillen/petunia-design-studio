# 19.01 — Layers Panel

# Identity

PanelId ptnd.panel.layers. Default: right dock in Design/Photo.

# Model

Virtualized hierarchical QAbstractItemModel backed by revisioned LayerTreeSnapshot/diffs keyed stable ObjectId. No QWidget per row.

# Row anatomy

Disclosure; thumbnail; type icon; name; effect/mask/symbol/link badges; optional target indicator; visibility; lock. Selection/hover/focus use separate token states.

# Selection

Single/multiple row selection maps to document SelectionState. Ctrl/Cmd/Shift follow platform tree conventions. Selecting mask thumbnail can set PixelTarget independently from parent object selection.

# Drag/drop

Drop zones distinguish reorder sibling, reparent into group, clip, mask attach and Surface move. Preview line/indent/badge shows exact semantic action before drop. Core validates cycle/read-only/symbol constraints.

# Actions

Create Layer/Group/Mask/Adjustment; Duplicate; Delete; Rename; Group/Ungroup; Lock/Unlock; Show/Hide; Rasterize/Expand; Locate resource; Select children; clipping/mask operations.

# Keyboard

Arrow tree navigation; Left/Right collapse/expand; Space visibility optional; F2 rename; Delete; modifier multi-select; context-menu key. Reordering has keyboard Actions as alternative to drag.

# Thumbnails

Async derived thumbnails; stale revision discarded; priority visible rows. Placeholder icons used under load/memory pressure.

# Search/filter

Filter by name/type/status/tag/visibility; filtering does not alter tree order. Search result can reveal ancestors context.

# Scale

10k+ rows must scroll interactively. Incremental diffs preferred over reset model.

# Persistence

Panel UI state (expanded groups/search/column preferences) workspace/session state, not document truth.

# Accessibility

Tree roles, level/expanded/selected, visibility/lock actions and badge descriptions.

# Tests

Huge tree, drag semantics, mask target, plugin object type, async thumbnail, missing resource, keyboard, diff after undo.