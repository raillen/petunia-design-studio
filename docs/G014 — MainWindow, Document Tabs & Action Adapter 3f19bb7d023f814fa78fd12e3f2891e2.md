# G014 — MainWindow, Document Tabs & Action Adapter

# Goal

Build functional Qt shell driven by semantic ActionRegistry.

# Depends

G005, G007, G013.

# Primary

ui-component-engineer + editor-engineer.

# Skills

lang-python, ui-implementation, editor-tooling, accessibility.

# Deliverables

QApplication bootstrap; MainWindow; menu bar; primary/context toolbar hosts; document tab controller; status bar; ActionAdapter ActionId->QAction; shortcut baseline; empty command palette; active WindowContext; unsaved close workflow using mock/minimal sessions.

# Acceptance

Actions enabled/disabled from semantic context; menu/shortcut/palette dispatch same ActionId; multiple tabs switch active session; dirty close Save/Discard/Cancel works against test service.

# Tests

pytest-qt semantic actions, shortcut conflict, tab reorder/close, focus, accessible names and multi-window smoke.

# Non-goals

Dock panels/canvas renderer/full tools.