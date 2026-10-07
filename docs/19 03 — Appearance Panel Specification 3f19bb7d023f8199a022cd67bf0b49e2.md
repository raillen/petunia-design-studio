# 19.03 — Appearance Panel Specification

# Identity

PanelId ptnd.panel.appearance. Primarily Design, available shared.

# Data

Ordered AppearanceEntry snapshot for selected drawable(s): fills, strokes, effects and object-level opacity/blend summary.

# Row

Type icon, label, swatch/summary, visibility, target indicator, optional disclosure for parameters.

# Selection

Exactly one active AppearanceEntry may be the editing target for Fill/Gradient/Stroke tools. Entry selection is view state, not document state.

# Actions

Add Fill, Add Stroke, Add Effect, Duplicate Entry, Remove, Reorder, Toggle, Copy/Paste Appearance, Clear Appearance.

# Drag

Reorders entries within legal stage constraints. If an effect cannot legally cross another stage, drop target disables with reason.

# Multi-selection

Show common appearance structure only when compatible; otherwise summary + explicit Apply/Add action rather than pretending independent stacks align.

# Keyboard

Arrow rows, Space toggle, Delete remove, Ctrl/Cmd+D duplicate, reorder shortcuts.

# Tests

Multiple fills/strokes, effects ordering, selected target syncing with gradient tool, mixed objects, undo and export fidelity.