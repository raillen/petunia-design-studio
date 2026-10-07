# 05.1 — QMainWindow Shell, Menus, Tabs, Toolbars, Status & Multi-Document Windows

# Main shell

QMainWindow is the host for menus, central document tab area, dock adapters, toolbars and status. It does not own document semantics.

# Document tabs

Use a dedicated DocumentTabController rather than coupling tab widgets to DocumentSession lifecycle. Tab close/move/detach maps to view/session actions.

# Menus/actions

QAction objects are generated/adapted from ActionRegistry metadata. Enabled/checked state derives from ActionAvailability snapshots. No menu callback contains document mutation logic.

# Toolbar

Petunia toolbar widgets are tokenized wrappers. Context toolbar uses ToolContextSchema and SelectionProperty schemas. Overflow is deterministic based on priority metadata.

# Status

Status bar aggregates ToolHint, selection summary, view toggles, jobs and zoom. Background jobs expose compact aggregate + expandable job center.

# Multi-document

Single app process can own many DocumentSessions and multiple windows/views. Each window has active view but shared registries/services.

# Platform menu

macOS global menu conventions and Windows/Linux in-window behavior supported without changing ActionId.

# Shutdown

Closing MainWindow calls ApplicationLifecycle service; Qt closeEvent itself cannot independently destroy sessions with unsaved data.