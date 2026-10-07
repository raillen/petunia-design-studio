# G14 — MainWindow, Tabs & QAction Adapter

# Goal

Build minimal PySide6 desktop shell backed by semantic actions.

# Depends

G06–G07, G13.

# Authority

05.1, 05.5, 08.2, 08.4.

# Owner

ui-component-engineer.

# Deliverables

QApplication bootstrap, MainWindow, document tab controller, menu bar, primary toolbar, status bar, ActionAdapter, active WindowContext.

# Acceptance

Dummy semantic actions appear consistently in menu/toolbar/palette; enable/checked state updates; tabs can open/close mock DocumentViews; no business logic in QAction callbacks.

# Evidence

pytest-qt/QtTest, keyboard/accessibility smoke.