# 09.28 — Plugin SDK, Safe Extension Contract, Capability UX & Runtime-Neutral API

<aside>
🧩

**Runtime-neutral contract with V1 binding resolved:** Aubrieta's Plugin SDK is defined semantically and remains independent from the host language. **Lua 5.5 + `mlua` is the accepted V1 scripting binding.** Wasmtime/WASI may host the same semantic SDK at the high-isolation tier. Python/JavaScript remain external MCP/client-SDK languages unless a future ADR adds another embedded binding. Runtime choice never redefines document, Action/Command, property, permission or UI semantics.

</aside>

# Plugin goals

The extension system must be simultaneously:

- easy enough for designers, technical artists and novice programmers;
- predictable enough for code agents to generate correct plugins;
- powerful enough for tools, effects, import/export, automation and data sources;
- safe enough that installing a plugin does not implicitly grant process/filesystem/network control;
- versionable without coupling plugins to Rust ABI or GPUI internals.

# Two-layer extension model

Separate **semantic Plugin SDK** from **runtime binding**.

**Layer A — V1 approachable scripting SDK: Lua 5.5 + `mlua`**

- actions/commands;
- document queries;
- typed properties;
- simple tools;
- declarative panels/forms;
- data-source adapters;
- import/export helpers;
- background jobs;
- plugin-scoped storage and brokered files/network.

**Layer B — high-isolation component tier: Wasmtime/WASI Component Model**

- optional/advanced tier for complex or less-trusted extensions;
- same capability broker, semantic registries and schemas;
- WIT bindings adapt the same API rather than creating a second business-logic universe.

V1 implementation may ship Layer A before Layer B. Absence of the WASM tier must not change Lua SDK semantics; adding Layer B later must pass semantic parity/conformance tests.

# Plugin package

Conceptual package:

```
my-plugin/
  plugin.toml
  src/
  resources/
    icons/
    strings/
  schemas/
  README.md
  LICENSE
```

Packaging format/extension is a later ADR. Manifest is authoritative for identity, compatibility and permissions.

# Manifest minimum

`plugin.toml` should describe:

- namespaced `plugin_id`;
- display TextId/name metadata;
- semantic version;
- Aubrieta compatibility range;
- plugin API version;
- runtime + runtime-version requirement;
- entry point;
- provided contributions;
- required/optional capabilities;
- requested permissions;
- bundled locale/resource packs;
- dependencies/conflicts;
- author/homepage/license/source;
- signature/hash metadata when distribution infrastructure exists.

# Beginner-first API shape

Provide a small, discoverable facade rather than forcing direct registry manipulation.

Conceptual scripting style:

```
plugin.action("org.example.center_selection")
  .title("text.center_selection")
  .when("selection.count > 0")
  .run(center_selection)
```

Exact syntax depends on runtime, but principles are stable:

- descriptive verbs/nouns;
- named parameters;
- explicit return/error types;
- no magic global mutable document;
- progressive path from simple helper APIs to advanced typed APIs.

# Canonical SDK domains

Expose documented facades such as:

- `app` — version/capabilities/context;
- `document` — immutable queries and transaction requests;
- `selection` — semantic selected IDs/summaries;
- `actions` — register/invoke semantic actions;
- `properties` — typed schema/query/edit;
- `tools` — normalized pointer/tool lifecycle;
- `ui` — declarative panels/forms/notifications only;
- `jobs` — background work/progress/cancel;
- `storage` — plugin-scoped storage;
- `files` — capability-brokered scoped file access;
- `network` — capability-brokered allowed requests;
- `resources` — TextId/IconId/help/resources;
- `import_export` — format contribution helpers;
- `data` — data-source/Data Merge adapters;
- `diagnostics` — structured logs/errors without arbitrary host tracing access.

# No mutable document handles

Plugins receive stable IDs, immutable snapshots and transaction APIs. Mutation always flows through validated Commands/Actions. Never expose raw Rust references, pointers, GPU resources or internal collection indices.

# Simple vs advanced API

Document both:

- **Simple API:** common helpers with safe defaults (`selection.set_fill`, `document.create_rectangle`, `export.current_surface`).
- **Advanced API:** schema-driven action execution, transactions, capability discovery and streaming/large-data access.

Simple helpers delegate to the same advanced semantic contracts so behavior cannot diverge.

# Declarative UI

Plugin UI uses host-rendered schemas/components. A plugin describes:

- fields/control hints;
- TextId/IconId/help metadata;
- validation;
- sections/tabs;
- action bindings;
- list/table data providers;
- empty/loading/error state;
- accessibility metadata.

The host renders using Aubrieta Design System. Plugins cannot inject raw GPUI components into the application process.

# Tool extensions

A plugin tool declares:

- `ToolId`;
- action/activation metadata;
- cursor semantic ID;
- accepted object/context capabilities;
- normalized pointer/pen phases;
- preview state;
- transaction/commit/cancel behavior;
- overlay descriptors where safe;
- property schema;
- permissions/resource budgets.

Canvas overlay API must be semantic/bounded, not unrestricted GPU command access in the scripting tier.

# Capability permissions

Default deny. Example scopes:

- `document.read`;
- `document.write`;
- `selection.read`;
- `ui.panel`;
- `clipboard.read` / `clipboard.write`;
- `filesystem.read:scoped`;
- `filesystem.write:scoped`;
- `network:https:<declared-hosts>`;
- `background.job`;
- `register.tool`;
- `register.importer`;
- `register.exporter`;
- `data_source.register`.

Permission names must be understandable in UI and docs.

# Permission UX

Install/enable dialog shows requested permissions grouped by risk:

- document access;
- local files;
- network hosts;
- clipboard;
- UI contributions;
- background execution.

Explain **why** each permission is requested when manifest provides rationale. Users can inspect/revoke grants later. Permission escalation after update requires renewed consent.

# Filesystem/network safety

Plugins do not get arbitrary process filesystem/network APIs through the SDK. Use brokers:

- file picker grants specific files/directories;
- scoped storage is isolated per plugin;
- write operations use atomic safe APIs where possible;
- network access is host/domain allowlisted from manifest/consent;
- redirects, response sizes and timeouts are bounded;
- credentials/environment secrets are never inherited automatically.

# Resource/time quotas

Runtime host can enforce/measure:

- memory budget;
- CPU/fuel/time slice where runtime supports it;
- concurrent jobs;
- open file handles;
- network requests;
- output/resource size;
- UI update frequency;
- document mutation batch size.

A plugin that exceeds a quota fails with structured diagnostics and cannot corrupt document state.

# Transactions

Plugin mutation is atomic by default:

```
begin transaction
  commands...
validate
commit -> one history transaction
or rollback
```

Long operations stage results outside canonical state and commit once valid. Plugins cannot leave partially mutated documents after exceptions/timeouts.

# Versioning

Version independently:

- plugin package schema;
- scripting SDK API;
- component/WIT API if used;
- contribution schemas;
- declarative UI schema.

Prefer additive evolution. Deprecations include machine-readable replacement metadata and documented removal window.

# Discovery and self-documentation

Runtime exposes `help()`, `capabilities()`, action/property/schema discovery and examples where practical. SDK reference is generated from the same schemas/source metadata used by the host to prevent drift.

# Error model

Errors contain:

- stable code;
- short human-readable title through TextId;
- explanation;
- offending parameter/path/ID when safe;
- suggested recovery;
- permission/capability requirement;
- optional docs/help topic.

Avoid stack traces as primary user/plugin-author feedback.

# Developer tooling

Ship templates/cookbook examples:

1. Hello Action;
2. document query;
3. transactional edit;
4. property inspector;
5. declarative panel;
6. custom tool;
7. Data Merge source;
8. importer/exporter;
9. background job;
10. permission-aware file/network task.

Provide validator, package checker and local dev reload workflow.

# Plugin testing

SDK should allow headless tests using a deterministic host fixture. Required host tests include denied permission, plugin exception, timeout, missing dependency, version mismatch, unload during job, document reopen without plugin, malformed schema, excessive output and rollback.

# Unload behavior

Disabling a plugin:

1. stop accepting new contribution invocations;
2. request cancellation of owned jobs;
3. finish/rollback active transactions;
4. detach panels/tools/actions;
5. persist plugin-owned settings safely;
6. preserve document extension data opaquely if provider absent;
7. release runtime resources.

Existing unrelated tools must continue functioning.

# Code-agent documentation rule

Every plugin API surface must include one minimal example, one failure/permission example, parameter schema, side effects, undo behavior, thread/job behavior, security notes and compatibility version. APIs without examples are considered incomplete for agent use.