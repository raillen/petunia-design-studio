# 01.2 — ApplicationCore, DocumentSession, ViewSession & Service Ownership

# ApplicationCore

Process-level semantic owner: registries, platform-independent application services, JobScheduler, open DocumentSession registry, plugin/MCP semantic façades. Does not own QWidget.

# DocumentSession

Owns one loaded canonical DocumentStore, HistoryManager, session revision/savepoint, resources/watchers, per-document derived services/cache namespace and jobs associated with document.

# ViewSession

A view/window/tab over DocumentSession with SelectionState, ViewportState, active Surface, active Persona/tool, overlays and view-only proof settings. Multiple ViewSessions may share one DocumentSession.

# Presentation

DocumentView Qt object binds one ViewSession. Panels generally resolve active ViewSession through explicit WindowContext.

# Lifetime

ApplicationCore > DocumentSession > ViewSession in logical ownership; jobs can hold safe snapshot/session tokens but cannot extend QWidget lifetime.

# Close

Closing a ViewSession does not necessarily close DocumentSession if another view/job owns it. Last view triggers dirty close policy. DocumentSession destruction cancels document jobs/watchers before DocumentStore.

# IDs

Session IDs are runtime IDs, never serialized as document identity.

# Tests

Duplicate view, close one, background export, dirty last view, app shutdown and no dangling bindings.