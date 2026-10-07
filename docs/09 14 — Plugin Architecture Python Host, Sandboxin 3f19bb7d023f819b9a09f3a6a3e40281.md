# 09.14 — Plugin Architecture: Python Host, Sandboxing, Permissions, SDK & Packaging

# Security premise

**Python cannot be safely sandboxed in-process as untrusted code.** Therefore third-party Python plugins run out-of-process by default.

# Tiers

**Tier A — built-in trusted Python:** shipped/reviewed with Petunia; in-process application layer.

**Tier B — third-party Python:** petunia-plugin-host subprocess, capability RPC, OS sandbox/restrictions, no direct app memory.

**Tier C — WASM component:** optional high-isolation runtime using same semantic SDK.

**Native C++ public plugin ABI:** deferred; internal modules only unless future ADR solves ABI/signing/isolation.

# Package

```
plugin.toml
src/
resources/icons/
resources/strings/
schemas/
README.md
LICENSE
```

# Manifest

plugin_id, version, Petunia/API compatibility, runtime, entry, contributions, required/optional capabilities, permissions + rationale, locales/resources, dependencies/conflicts, author/license/source, package hashes/signature metadata.

# Semantic SDK

app, document, selection, actions, properties, tools, ui, jobs, storage, files, network, resources, import_export, data, diagnostics.

# No raw handles

Plugins receive IDs/snapshots/schemas. Mutations go through transactions/Actions. No raw C++ pointer, QWidget, GPU command buffer, unrestricted Python object reference.

# Declarative UI

Plugin defines panel/form schema, TextId/IconId, validation, actions, list providers, loading/error/empty/accessibility. Host renders Petunia controls. Tier B cannot inject QWidget into main process.

# Tools

Plugin ToolId receives normalized pointer phases and semantic overlay descriptors. Heavy custom computation may use plugin process; canvas preview frequency is throttled/budgeted.

# Permissions

[document.read/write](http://document.read/write), [selection.read](http://selection.read), ui.panel, register.tool/importer/exporter/data, clipboard read/write, filesystem scoped read/write, network allowlisted HTTPS, background jobs, plugin storage.

# Broker

File picker grants opaque handles/scopes; network broker applies hosts, redirects, size/time limits; environment/secrets are not inherited.

# Quotas

Memory/process CPU, concurrent jobs, message rate, open handles, network requests, output size, mutation batch size, UI updates.

# Lifecycle

install -> verify manifest/package -> show permissions -> enable/start host -> handshake versions/capabilities -> contributions -> health monitor -> disable/terminate. Crash isolates plugin and does not corrupt document.

# Versioning

Package schema, semantic SDK, UI schema and RPC version evolve separately. Deprecations machine-readable with window.

# Developer UX

CLI/template, type stubs, schema completion, examples, test host, permission simulator, logs and compatibility checker.

[09.14.1 — Plugin Package, Manifest, Python SDK, Contributions & Examples](09%2014%201%20%E2%80%94%20Plugin%20Package,%20Manifest,%20Python%20SDK,%20Co%203f19bb7d023f81a893d9f653e682aa93.md)

[09.14.2 — Plugin Host Process, RPC, Sandbox, Quotas, Crash Isolation & Permissions](09%2014%202%20%E2%80%94%20Plugin%20Host%20Process,%20RPC,%20Sandbox,%20Quota%203f19bb7d023f81d493b1d58b9113120f.md)

[09.14.3 — Plugin Manager GUI, Permission UX, Declarative Panels & Developer Tooling](09%2014%203%20%E2%80%94%20Plugin%20Manager%20GUI,%20Permission%20UX,%20Decla%203f19bb7d023f81fcafbbfa1ebc128e45.md)

[09.14.4 — Plugin RPC Protocol: Handshake, Messages, Framing, Errors & Versioning](09%2014%204%20%E2%80%94%20Plugin%20RPC%20Protocol%20Handshake,%20Messages,%203f19bb7d023f81409609c5730663681b.md)

[09.14.5 — Plugin Capability & Permission Semantics: Exact Grants, Lifetimes & Revocation](09%2014%205%20%E2%80%94%20Plugin%20Capability%20&%20Permission%20Semantics%203f19bb7d023f815a8e44c2a706b790a2.md)

[09.14.6 — Declarative Plugin UI Schema: Panels, Forms, Lists, Actions & Accessibility](09%2014%206%20%E2%80%94%20Declarative%20Plugin%20UI%20Schema%20Panels,%20For%203f19bb7d023f81bf8f03c6e082196612.md)

[09.14.7 — Plugin SDK Python API Surface, Type Contracts & Reference Examples](09%2014%207%20%E2%80%94%20Plugin%20SDK%20Python%20API%20Surface,%20Type%20Cont%203f19bb7d023f815ca231eb36ed22c129.md)

[09.14.4 — Plugin RPC Wire Protocol, Message Envelope, Versioning & Error Codes](09%2014%204%20%E2%80%94%20Plugin%20RPC%20Wire%20Protocol,%20Message%20Envelo%203f19bb7d023f8147913fee8645e9b98d.md)

[09.14.5 — Plugin Declarative UI Schema, Panel Models, Actions & Accessibility](09%2014%205%20%E2%80%94%20Plugin%20Declarative%20UI%20Schema,%20Panel%20Mode%203f19bb7d023f81e689b0d2922463aae6.md)