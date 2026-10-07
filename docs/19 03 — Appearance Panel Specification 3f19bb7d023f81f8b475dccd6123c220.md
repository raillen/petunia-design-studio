# 19.03 — Appearance Panel Specification

# Identity

PanelId ptnd.panel.appearance.

# Model

Ordered AppearanceEntry list for active object/selection. Entry kinds Fill, Stroke, Effect, opacity/blend group if represented.

# Interaction

Select entry determines target of Color/Stroke/Gradient controls. Add Fill/Stroke/Effect buttons, duplicate, remove, visibility, drag reorder where semantic.

# Multi-selection

Show common appearance structure only when safely alignable; otherwise summarize Mixed and offer Add common entry action rather than pretending indexes correspond.

# Drag

Reordering validates effect/appearance constraints. Preview line shows insertion. Commit one ReorderAppearanceEntry.

# Inline summary

Fill swatch/gradient icon, stroke width/style, effect type and enabled state.

# Context

Copy/Paste Appearance uses semantic fragment; Paste can replace or append explicitly.

# Accessibility/tests

List roles, move up/down keyboard actions, entry target announced. Test multiple fills/strokes, mixed selection, effect reorder and color panel target sync.