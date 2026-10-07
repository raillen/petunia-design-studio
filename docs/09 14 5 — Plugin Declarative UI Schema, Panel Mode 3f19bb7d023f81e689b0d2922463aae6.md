# 09.14.5 — Plugin Declarative UI Schema, Panel Models, Actions & Accessibility

# UI schema

Plugin UI is host-rendered from versioned declarative component tree:

Section, Label, Button(ActionId), Toggle(Property/action), NumericField, TextField, Combo, List/Table model, ColorWell, Progress, Notice, Separator and supported composite controls.

# Identity

Every control has stable plugin-scoped ControlId, TextId label/help and optional PropertyId/ActionId binding.

# State

Plugin process supplies immutable PanelViewModel revision: values, enabled/visible, loading/error, list page cursors. Host diffs and renders Qt controls.

# Events

User interaction sends semantic UIEvent {ControlId, eventKind, value, viewModelRevision}. Plugin may respond with new model or Action request. Stale model edits rejected/reconciled.

# Lists

Paginated model with stable RowId; sorting/filtering request semantics declared. No QWidget-per-row or arbitrary HTML.

# Styling

Plugin may select semantic emphasis/icon IDs but not arbitrary QSS/fonts/window chrome. Content preview canvas requires separate capability.

# Accessibility

Label/description required for interactive controls. Host validates missing accessible metadata and maps controls to QAccessible automatically.

# Tests

Theme/density, localization expansion, stale model, 10k rows, plugin crash while panel open and keyboard-only interaction.