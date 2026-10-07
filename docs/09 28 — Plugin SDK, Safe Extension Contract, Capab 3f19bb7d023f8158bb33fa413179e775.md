# 09.28 — Plugin SDK, Safe Extension Contract, Capability UX & Runtime-Neutral API

# Canonical SDK

Plugin contract is semantic and runtime-neutral even though Python is primary authoring language.

# Beginner facade

Examples should read as document.create_rectangle, selection.set_fill, app.register_action, ui.register_panel rather than direct registry internals. Helpers delegate to typed primitive layer.

# Advanced layer

Schema discovery, explicit transactions, immutable snapshots, pagination/streaming resources, JobId and capability negotiation.

# Contributions

Actions, commands only through host semantic requests, panels/forms, tools, importers/exporters, data sources, effect providers where runtime/performance contract permits, resources/help.

# Python type package

petunia_plugin_sdk ships full type hints, enums/dataclasses/protocols and generated schemas. Unsupported host version fails handshake before plugin executes meaningful contribution.

# Declarative UI

Panel schemas use host component IDs and Property schemas. Third-party process sends state/model updates, never QWidget code.

# Permission UX

Install/update dialog groups Document, Files, Network, Clipboard, UI/Tools, Background and Administration permissions; rationale displayed; user can revoke later.

# SDK docs

Every capability: what/when, minimal/advanced example, permissions, transactions/undo, performance limits, errors/recovery, version introduced/deprecated.

# Conformance

Reference plugins: Hello Action, transactional edit, property panel, custom tool, CSV-like data source, exporter, background job and permission denial.