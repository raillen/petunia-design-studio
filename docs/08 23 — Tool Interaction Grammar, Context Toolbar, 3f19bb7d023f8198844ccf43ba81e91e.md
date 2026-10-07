# 08.23 — Tool Interaction Grammar, Context Toolbar, Modifiers & Canvas HUD Contract

# Universal phases

Inactive -> Hover -> Armed -> Gesture/Editing -> Commit or Cancel. Tools may add substates but must map Escape and document/window interruption explicitly.

# Activation

Click tool, shortcut or Action. Re-press grouped-tool shortcut cycles according preference. Temporary override key returns prior tool on release.

# Pointer capture

Gesture captures pointer safely; leaving canvas/window does not lose final up/cancel. Lost capture triggers deterministic cancel/recovery.

# Modifiers

Shift generally constrains/adds, Alt/Option generally center/duplicate/subtract, Ctrl/Cmd platform semantics. Exceptions must be shown in status hint and documented; do not overload five meanings invisibly.

# Context bar

Control IDs and property bindings are declared by tool schema. Stable family order: mode -> geometry/target -> appearance -> precision -> advanced.

# HUD

Display only immediate gesture information. Numeric entry may focus HUD via shortcut but advanced settings remain panel. HUD never becomes undocumented mini-application.

# Commit

One logical history transaction. Tool stays active unless action semantics naturally switch to selection according preference.

# Cancel

Esc cancels most recent uncommitted sub-operation first, then exits editing mode only on subsequent Esc when appropriate.

# Accessibility

Every drag operation has keyboard/numeric equivalent for final precision where reasonable. Status bar verbalizes modifiers and current target.