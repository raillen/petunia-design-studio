# 09.1 — Modularity, Capability Registry & Contribution Architecture

# Goal

Aubrieta is assembled from capabilities instead of hard-coded feature-to-feature dependencies. Built-in features and extensions contribute through stable registries.

# Canonical concepts

`ModuleId`, `CapabilityId`, `ContributionId`, `ProviderId`, `ModuleManifest`, `CapabilityRequirement`, `ContributionRegistry`, `ModuleRuntimeState`.

A module declares **requires**, **optional_requires**, **provides**, lifecycle hooks and contributions.

# Contribution kinds

- Action;
- Tool;
- Panel/Inspector;
- Menu/Command Palette entry;
- Effect/Adjustment/Generator;
- Importer/Exporter;
- DataSource adapter;
- Resource pack provider;
- File handler;
- Diagnostics provider;
- optional workspace/persona contribution.

# Dependency rules

A module depends only on domain/service contracts it requires. It must not call another feature module by concrete type. Optional integrations query a capability handle and degrade if unavailable.

Circular required dependencies are invalid at registry construction. Optional cycles must not create initialization ordering requirements.

# Lifecycle

`Discovered → Validated → Resolved → Initialized → Active → Suspended/Disabled → Shutdown`.

Initialization is transactional: failed registration rolls back all contributions from that module. Disable removes its UI/action contributions only after active operations safely quiesce.

# Runtime disable/removal

Core built-ins may be compile-required yet runtime-hideable. Truly detachable modules must document what happens to document objects they own when disabled: preserved opaque data, fallback renderer, read-only placeholder or explicit incompatibility. Never silently delete unsupported data.

# Collision policy

Contribution IDs are namespaced. Duplicate canonical IDs fail deterministically with source diagnostics. User overrides can replace presentation resources but cannot replace domain behavior unless an explicit override contract allows it.

# Persona composition

Design/Photo are capability presets. A Persona is a list of visible tools/panels/actions and defaults; it does not own implementations. User workspaces may mix capabilities.

# Testing

Test missing optional provider, disable/re-enable, registration rollback, duplicate IDs, version mismatch, invalid manifests, module-owned document data without provider and deterministic capability resolution.

# Extensible identifiers — implementation rule

Do **not** model registries as closed enums such as `enum ActionId { Export, Pen, ... }`, because that prevents external modules from contributing IDs without recompiling the core. Canonical IDs are validated namespaced strings/newtypes/interned atoms, for example:

```rust
ActionId("aubrieta.action.export")
ToolId("aubrieta.tool.pen")
PanelId("aubrieta.panel.layers")
CapabilityId("aubrieta.capability.data_merge")
ActionId("org.example.plugin.smart_align")
```

Built-in IDs should be generated as constants (`actions::EXPORT`, `tools::PEN`) to keep call sites typed and typo-safe while preserving runtime extensibility.

The same namespacing strategy applies to `TextId`, `IconId`, `PropertyId`, `SettingId`, `EffectTypeId`, importer/exporter IDs and schema namespaces. Registry validation enforces syntax, namespace ownership and collision policy.

# Strong modularity contract

Aubrieta treats modularity as **runtime and architectural behavior**, not merely Cargo crate organization. A module that is optional in product composition must be detachable without requiring neighboring feature code to know its concrete type.

## Module manifest contract

Every feature module declares:

- `ModuleId` and semantic version;
- required and optional capabilities;
- capabilities it provides;
- contribution IDs;
- owned settings/resource namespaces;
- document extension data it owns, if any;
- migration responsibility;
- background jobs it may spawn;
- permission requirements;
- thread/executor assumptions;
- startup/shutdown hooks;
- disable/unload behavior;
- diagnostics owner;
- test fixtures and representative failure modes.

## Dependency classes

Use three classes explicitly:

1. **Foundational dependency** — unavoidable semantic core such as IDs/document contracts;
2. **Required capability dependency** — feature cannot operate without the capability but still depends on its interface, not implementation;
3. **Optional integration** — queried dynamically and omitted cleanly when absent.

Do not convert optional integration into a required crate dependency merely for implementation convenience.

## No sideways feature calls

Feature A may not directly call Feature B's implementation object. Cooperation happens through:

- domain/application ports;
- capability interfaces;
- Action/Command dispatch;
- typed events/change notifications;
- schema/registry contributions.

This prevents dependency webs that make one tool impossible to remove.

## Built-in equals extension principle

Where practical, built-in tools/panels/effects/importers register through the same contribution registries as external modules. Built-ins may have privileged performance access internally, but their semantic identity, lifecycle and discovery must not require a second parallel API.

## Detach proof

Optional capability work is complete only when CI can disable or remove the module and verify:

- unrelated modules still initialize;
- document opens without data loss;
- missing-provider state is explicit;
- menus/panels/actions remove themselves cleanly;
- persisted opaque extension data round-trips when required;
- no dangling jobs/listeners/handles remain;
- saved workspace gracefully repairs missing panels;
- MCP/plugin discovery no longer advertises absent capability.

## Unload sequence

Canonical safe order:

1. stop new invocations;
2. mark capability unavailable for discovery;
3. cancel/quiesce owned jobs;
4. finish or rollback active transactions;
5. detach UI/action contributions;
6. flush module settings/state;
7. preserve or migrate module-owned document payloads according to contract;
8. unregister capabilities/resources;
9. release runtime resources.

Unloading one module must not force a process restart unless the module explicitly documents a platform/runtime limitation.

# Core/interface separation

A capability contract must never embed GPUI types. UI contributions describe semantic panel/tool/action metadata and are rendered by the active shell adapter. See 09.27 and 08.21.

# Code-agent rule

When an agent adds a feature, it must answer: **Can this feature be disabled or replaced without modifying unrelated feature logic?** If the answer is no, the agent must either classify the dependency as foundational with justification or redesign the boundary before completion.