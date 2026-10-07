# 19.01 — Layers Panel Specification

# Identity

PanelId ptnd.panel.layers. Shared Design/Photo. Default right dock, high priority.

# Data model

QAbstractItemModel adapter over revisioned layer/object hierarchy snapshot. Rows keyed by stable ObjectId, never raw domain pointers.

# Row anatomy

Disclosure, thumbnail, type icon, name, status badges, visibility, lock, mask/effect/link indicators. Row height/density tokenized.

# Selection

Panel selection synchronizes with document/view selection through SelectionService without feedback-loop duplicate commands. Multi-select supports Shift range and Ctrl/Cmd toggle.

# Keyboard

Up/Down navigate; Left/Right collapse/expand; Space toggle visibility by profile; F2 rename; Delete remove; shortcuts for group/duplicate; full context menu reachable by keyboard.

# Drag/drop

Drop zones distinguish reorder-before/after, nest, clip and mask. Model computes proposed semantic operation and displays insertion preview. Invalid cycles/locked targets rejected before drop.

# Virtualization

10k+ rows must scroll without QWidget-per-row. Thumbnails load async and stale requests cancel.

# Search/filter

Optional search by name/type/tag and filters for hidden/locked/type. Filtering never changes canonical order.

# Actions

Rename, Visibility, Lock, Group/Ungroup, Duplicate, Delete, Reparent/Reorder, Create Mask, Release Clip, Convert/Rasterize where applicable.

# Accessibility

Tree role, hierarchy level, expanded state, selected, visible/locked status and row actions exposed.

# Tests

10k hierarchy, deep nesting, multi-selection, drag clip/nest distinction, plugin object type missing, async thumbnails, keyboard-only workflows.