# G017 — Layers QAbstractItemModel & Semantic Drag/Drop

# Goal

Implement scalable Layers panel model over semantic snapshots/diffs.

# Depends

G004, G005, G014, G015.

# Primary

ui-component-engineer + editor-engineer.

# Skills

lang-python, editor-tooling, ui-implementation, accessibility.

# Deliverables

LayerTreeSnapshot facade; QAbstractItemModel keyed stable IDs; roles for name/type/visible/locked; multi-selection sync; inline rename; visibility/lock Actions; semantic drag MIME; reorder/reparent Commands; async thumbnail placeholder architecture.

# Acceptance

10k-row synthetic tree scrolls responsively; reorder/reparent validates through commands; undo updates model incrementally; no raw native pointer stored in QModelIndex.

# Tests

Hierarchy diffs, rename, multi-select, cycle rejection, drag/drop, keyboard tree navigation and accessibility roles.

# Non-goals

Masks/effect badges full richness; production thumbnails.