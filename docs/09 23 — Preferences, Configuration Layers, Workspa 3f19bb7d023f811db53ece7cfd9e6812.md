# 09.23 — Preferences, Configuration Layers, Workspace State & Persistence Semantics

# Scopes

Built-in defaults -> platform -> user -> workspace -> document semantic config. Each setting declares scope and whether sync/exportable.

# Formats

Human-readable TOML/JSON for app/workspace settings with schema version. Secrets never stored there.

# Preferences

UI, tools, performance, color defaults, files/recovery, shortcuts, plugins, MCP, accessibility, updates.

# Workspace

Dock tree, visible panels, floating geometry, Persona preset, toolbar layout, density/theme overrides. Workspace does not contain artwork/document selection unless explicitly view-session restoration.

# Session state

Recently open docs, last active tab, viewport recovery are separate ephemeral/session store to avoid polluting preferences.

# Migration

Stepwise schema migration with backup/fallback. Unknown optional fields preserved when practical; corrupt preferences reset narrowly with diagnostic.

# Atomic write

Settings/workspace writes atomic similar safety helper but lower criticality than document save.

# Portable export

Workspace/shortcuts/theme profile can export/import without secrets, absolute private paths or plugin grants by default.