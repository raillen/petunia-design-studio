# 19.07 — Transform & Align Panels Specification

# Transform

PanelId ptnd.panel.transform. Fields X/Y/W/H, rotation, shear, anchor grid, aspect lock, scale strokes/effects and coordinate space.

# Input

Expressions/units allowed. Relative operations if syntax defined. Scrub labels preview/commit one transaction.

# Multi-selection

Values can represent aggregate bounds or per-object transforms; mode must be explicit. Editing W/H scales selection frame, not assigns each object same width unless alternate command.

# Align

PanelId ptnd.panel.align. Target Selection/Key Object/Surface/Margins. Buttons Left/Center/Right/Top/Middle/Bottom, distribute centers/edges and equal spacing.

# Key object

Selected explicitly; UI highlights key. Alignment keeps it fixed.

# Accessibility/tests

All icon actions have labels/shortcuts. Test rotated selection, mixed parents, key object, expressions and undo.