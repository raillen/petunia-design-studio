# G17 — Layers QAbstractItemModel & Selection Synchronization

# Goal

Build scalable Layers panel projection.

# Depends

G04–G07, G14–G15.

# Authority

05.6, 19.01.

# Owner

ui-component-engineer + editor-engineer.

# Deliverables

LayerTreeSnapshot/diff API, QAbstractItemModel, selection sync, visibility/lock actions, inline rename, drag reorder/nest baseline and async thumbnail placeholder protocol.

# Acceptance

10k+ rows scroll without widget-per-row; row IDs stable across diff; hierarchy operations use Commands; keyboard navigation/accessibility works.

# Evidence

large-tree benchmark, drag/drop tests, semantic UI tree.