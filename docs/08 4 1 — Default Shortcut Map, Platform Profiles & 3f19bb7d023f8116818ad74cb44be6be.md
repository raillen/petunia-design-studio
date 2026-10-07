# 08.4.1 — Default Shortcut Map, Platform Profiles & Conflict Rules

# Profiles

Windows/Linux share Ctrl-based defaults where conventions align; macOS uses Cmd/Option equivalents. Tool single-key shortcuts are profile-stable unless system conflict.

# Core file/edit

New, Open, Close, Save, Save As, Export, Undo, Redo, Cut/Copy/Paste, Duplicate, Select All, Deselect, Preferences, Command Palette and Help.

# View

Zoom In/Out, 100%, Fit Surface, Fit Selection, Show/Hide UI, rulers/guides/grid/snapping toggles and panel focus cycling.

# Tools

V Move, A Node, P Pen, T Text family, G Fill/Gradient, B Pixel Brush, E Eraser, selection group keys and Space temporary Hand baseline; exact defaults live in machine ShortcutProfile.

# Chords

Multi-stroke chords allowed only if discoverable and conflict resolver understands prefix ambiguity. Single-letter tools disabled while text caret has focus unless explicit temporary behavior.

# Conflict

Exact duplicate and prefix conflicts block assignment or require user override resolution. OS-reserved shortcuts cannot be silently stolen.

# Context

Text editor, modal dialog and tool gestures can shadow global actions only through documented priority stack.

# Tests

All built-in ActionIds have valid/default/no-conflict profile; macOS remap; plugin ActionId conflict and text-focus suppression.