# 09.14.1 — Plugin Package, Manifest, Python SDK, Contributions & Examples

# Package layout

```
my-plugin/
  plugin.toml
  src/
    plugin.py
  resources/
    icons/
    strings/
  schemas/
  tests/
  README.md
  LICENSE
```

# Manifest conceptual fields

```toml
plugin_id = "org.example.smart-export"
version = "1.2.0"
petunia = ">=1.0,<2.0"
sdk = "1"
runtime = "python"
entry = "src.plugin:activate"

[contributions]
actions = ["org.example.smart_export"]
panels = ["org.example.smart_export.panel"]
exporters = []

[permissions]
document_read = true
document_write = false
ui_panel = true
filesystem_write_scoped = true
network_hosts = []
```

Exact schema is versioned and generated; example is illustrative.

# Activation API

Host passes PluginContext with app/version/capabilities, registration facade and scoped services. Plugin returns ContributionSet/activation handle. No module-global mutable document.

# Action example

Plugin registers namespaced ActionId, title TextId, availability predicate/schema and handler. Handler receives immutable context/snapshot and returns semantic transaction request or non-mutating result.

# Panel

Declare PanelSchema sections/fields/tables/actions. Data provider responds with typed models/page cursors. UI runs in main app using Petunia components; plugin process only provides data/intents.

# Tool

Register ToolDescriptor: cursor, accepted targets, context properties, event subscriptions, overlays and commit Actions. High-frequency events are coalesced; plugin cannot issue raw draw commands.

# Import/export

Plugin adapter reads/writes only via granted stream/file handle. Capabilities/fidelity descriptor mandatory. Parser budgets and cancellation enforced.

# Version checks

Handshake validates package schema, Python runtime support, SDK version and required host capabilities before enabling contributions.

# Type support

SDK ships py.typed/type stubs. Plugins recommended Pyright strict. Runtime validates messages despite static hints.

# Examples shipped

hello_action, selection_transform, property_panel, simple_tool, csv_data_source, exporter, background_job, permission_denied test.