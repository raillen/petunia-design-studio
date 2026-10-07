# 19.06 — Transform & Align Panels Specification

# Transform Panel

PanelId ptnd.panel.transform. Shared.

# Controls

X/Y/W/H, rotation, shear/skew, 3x3 anchor, aspect lock, coordinate space selector and scale stroke/effects.

# Numeric semantics

Expressions/units accepted. Relative operations can use +=/-= style only if parser spec supports. Invalid input never mutates.

# Multi-selection

Fields operate on selection bounds or individual objects according explicit transform mode. Mixed transform values distinguished from selection-bounds aggregate.

# Align Panel

PanelId ptnd.panel.align. Target mode Selection, Key Object, First/Last, Surface, Margins where valid.

# Actions

Horizontal/vertical align, distribute centers/edges, equal spacing, set exact gap and optional distribute-to-Surface.

# Preview

Optional hover preview is view-only. Key object highlighted.

# Keyboard/accessibility

All functions are QAction-backed and command-palette searchable.

# Tests

Rotated objects, nested transforms, key object, mixed parents, exact spacing, units and undo.