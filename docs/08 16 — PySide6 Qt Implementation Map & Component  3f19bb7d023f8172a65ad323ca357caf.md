# 08.16 — PySide6/Qt Implementation Map & Component Ownership

# Python packages

```
petunia_app/
  bootstrap.py
  application.py
  actions/
  tools/
  panels/
  dialogs/
  workspace/
  models/
  design_system/
  platform/
  plugins/
  mcp/
```

# Ownership

MainWindow = shell.

DocumentView = session + viewport.

CanvasHost = native surface/input.

PanelHost = provider/lifecycle.

ActionAdapter = ActionRegistry to QAction/QShortcut.

QtModel adapters = snapshot/diff to QAbstractItemModel.

ThemeService = tokens to Qt.

AccessibilityBridge = semantic tree to QAccessible.

# Typing

Public constructors/methods/signal payloads typed. Protocol/TypedDict/dataclass/Enum. Evitar Any; external JSON schemas são validados/generated.

# Threads

Widgets somente main thread. Native jobs completam via thread-safe dispatcher; Python callbacks na main thread.

# UI state

QObject pode possuir ephemeral state. Persisted workspace/preferences usam typed pure-Python models.

# Model/view

Layers, Assets, History, Data Merge e Preflight usam QAbstractItemModel; sem QWidget por row.

# Canvas

QWidget/QWindow fornece surface/handle; C++ renderer possui frame resources.

# Error boundary

Top-level UI event catches unexpected Python exception -> structured diagnostic + safe notification; canonical mutation continua protegida por transaction.