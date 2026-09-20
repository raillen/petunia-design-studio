# 09.23 — Preferences, Configuration Layers, Workspace State & Persistence Semantics

# Separate state classes

Do not store everything in one settings blob.

- **Application preferences:** user-global behavior (theme selection, units defaults, autosave policy, recent limits);
- **Workspace state:** docking/panel layout, visible panels, Persona composition, toolbar customization;
- **Document preferences:** creative semantics that travel with document (working profile, units if document-specific, grid/guides stored where semantic);
- **Session state:** transient selection, viewport, dialog state;
- **Machine state:** GPU/backend cache, window placement/monitor association, local paths.

# File formats

Human-authored/user-editable config uses TOML. Workspace layout may use JSON if produced structurally by docking engine, but the outer workspace manifest remains TOML and schema-versioned. Never mix document semantics into user config.

# Precedence

Built-in defaults → platform defaults → user preferences → workspace preset → document-specific semantic settings → transient session overrides. Each setting declares which layers are legal.

# Typed settings registry

`SettingId` is namespaced/extensible. `SettingDescriptor` declares type/schema, default, scope, TextId, optional IconId, restart requirement, live-preview policy, validation and module owner. Preferences UI is driven by registry rather than hard-coded switch statements.

# Unknown settings

Preserve namespaced plugin/module settings when provider temporarily unavailable, subject to size/security limits. Never reinterpret unknown values.

# Writes

Debounced/atomic config writes with backup/replace strategy. Invalid manual edit reports line/key diagnostics and retains last-known-good snapshot; do not overwrite invalid file immediately.

# Reset/import/export

Reset one setting/section/all; export/import preference/workspace pack with schema validation and conflict summary. Secrets/permissions are excluded from normal export.

# Roaming

**Cloud synchronization is OUT_OF_SCOPE for V1.** A later product decision may promote a dedicated sync subsystem only through an accepted persistence/conflict/security ADR. Config schema must therefore not assume paths, monitor IDs or other machine-local values are portable or synchronizable.

# Tests

Manual syntax error, old schema migration, plugin missing, invalid value fallback, live theme switch, workspace layout corruption, multi-window placement on missing monitor, concurrent config writes.

# Setting scope is part of schema

Every `SettingDescriptor` declares exactly one primary scope plus allowed override scopes. A feature may not read “whatever setting is available” dynamically.

Canonical scopes:

```
BuiltInDefault
PlatformDefault
UserPreference
WorkspacePreset
DocumentSemanticSetting
SessionOverride
MachineLocalState
SecurityGrantStore   # separate protected store, never ordinary preference export
```

`DocumentSemanticSetting` is only for values whose meaning must travel with the creative document; it is serialized through the document schema, not the application settings file.

# Precedence resolution

Resolution is deterministic per setting descriptor. The global conceptual order is:

```
built-in
→ platform default
→ user
→ workspace
→ document semantic override (only for descriptors that permit it)
→ session override
```

Machine-local state is not a generic higher-precedence layer; it owns separate settings such as monitor/window placement. Security grants are separately queried and cannot be overridden by theme/workspace TOML.

# Canonical configuration files

Recommended V1 layout under platform config/data roots:

```
config/
  preferences.toml
  keymap.toml
  recent.toml-or-derived-state
  workspaces/
    <workspace-id>/manifest.toml
    <workspace-id>/layout.json
  modules/
    <module-id>.toml
state/
  machine-state.toml
  window-state.toml
security/
  grants.db-or-protected-store   # implementation-specific protected storage
```

Exact OS paths come from `aubrieta_platform`. User-facing export never blindly copies the entire config directory.

# Preferences TOML schema

Top-level file carries explicit `schema_version`. Settings are namespaced/stable, for example:

```toml
schema_version = 1

[ui]
theme = "system"
density = "comfortable"
icon_family = "lucide"

[document.defaults]
units = "mm"

[autosave]
enabled = true
interval_seconds = 120
```

Implementation deserializes into typed schema/layer structures and retains unknown newer namespaced keys separately for round-trip where safe.

# Setting value model

Descriptors use typed values, not arbitrary strings. Supported schema types include bool, integer, finite float, enum, string, path/grant reference when legal, duration, size/count, semantic ID and structured nested schema.

Validation runs before a value becomes active. Invalid override falls back to the next valid lower-precedence layer and produces a diagnostic identifying the key/source.

# Default ownership

Defaults live with the owning descriptor/module, not duplicated between Rust `Default`, Preferences UI and TOML template. Generated example/default config is derived from descriptors to prevent drift.

# Unknown settings preservation

When a newer/plugin-owned key is unknown:

- preserve raw TOML-compatible value under its namespace if within size/depth limits;
- do not expose it as active typed value;
- do not move it into another namespace;
- restore normal interpretation when provider returns;
- explicit Reset All may remove unknown user keys only if the confirmation states this behavior; default reset should preserve third-party opaque config when possible.

# Corrupt/invalid file recovery

Load pipeline:

1. read bounded file;
2. parse TOML;
3. validate schema/version;
4. migrate known historical schema;
5. validate each setting;
6. publish immutable `ConfigSnapshot`.

If syntax is invalid, keep last-known-good in-memory/snapshot config active. Do **not** overwrite the user's invalid file. Write a `.diagnostic`/UI warning and offer `Open File`, `Restore Last Known Good`, `Reset`, or `Save Repaired Copy` where appropriate.

# Atomic configuration snapshot

Like resources, effective settings are published as immutable generation snapshots. A multi-setting Preferences Apply transaction validates the full change then atomically publishes generation N+1. Observers cannot see half-applied theme/density/keymap changes.

# Live preview transaction

Settings marked `live_preview=true` use:

```
BeginPreferencePreview
→ ApplyPreview(value*)
→ Commit
  or Cancel/restore exact pre-preview snapshot
```

Preview changes are session-only until Apply/commit. Cancel never rewrites persistent files with intermediate values.

# Restart-required settings

Descriptor can state `ApplyMode = Live | Commit | RestartSubsystem | RestartApplication`. UI shows reason/affected subsystem. Persisted value may be accepted now while current effective runtime value remains old until restart; APIs expose both configured/effective state so automation/UI does not lie.

# Workspace identity/model

Workspace is a versioned named composition, not one opaque docking blob:

```
WorkspacePreset
- WorkspaceId
- name Text/UserLabel
- schema_version
- persona affinity/default
- enabled panel/tool/action contribution IDs
- toolbar/tool-rail customization
- layout reference
- density/theme override only if explicitly configured
- monitor-placement policy
- module requirements/optional panel IDs
```

Docking engine layout JSON is subordinate structural data and must reference stable `PanelId`/instance IDs, never GPUI entity handles.

# Missing panel/module repair

Opening a workspace whose plugin/panel is unavailable:

- preserve unknown panel placement payload opaquely if feasible;
- remove it from active layout without breaking sibling split tree;
- surface a nonblocking missing-contribution diagnostic;
- if provider returns later, offer/perform deterministic restoration according to workspace policy.

Workspace load must always have a safe fallback layout.

# Built-in workspace immutability

Built-in presets are immutable templates. “Update current” on built-in creates/updates a user override/copy, not modifies application-shipped resource. `Reset Built-in` discards user override and reconstructs from bundled canonical preset.

# Workspace serialization portability

Monitor IDs, pixel coordinates and OS-specific window flags are machine-local annotations, not portable workspace semantics. Exported workspace pack contains logical dock/panel composition and normalized geometry; imported layout maps/clamps to current displays.

# Window/machine state

Window placement is separate from workspace content because it can be invalid on another machine. Store display association, normalized/physical geometry and scale metadata sufficient to clamp/recover if monitor disappears.

# Keymap

Keymap is a typed settings subsystem referencing `ActionId` + `KeyContextId`, not localized command names. Import/export validates unknown actions/contexts, conflicts and platform-specific modifiers. Missing plugin action bindings are preserved opaquely where safe.

# Recent files

Recent-file list is machine-local derived convenience state. It stores native path/file identity metadata and optional thumbnail reference, never becomes document/user-workspace semantics. Missing entries remain removable and do not trigger network/file scanning at startup beyond bounded policy.

# Secrets and grants

API tokens, credentials, signing secrets, network auth and plugin permission grants are **never stored in ordinary TOML preference/workspace exports**. Use OS credential/protected storage or a dedicated security store through `aubrieta_platform`/security service. Preference UI references grant IDs/status, not secret values.

# Import/export of preferences

A preference pack has explicit selectable scopes and preview diff:

- appearance/theme choice;
- keymap;
- tool/canvas preferences;
- workspaces;
- document defaults.

Excluded by default and normally prohibited: recent files, native paths that are machine-specific, recovery state, credentials, plugin permission grants, crash/support data.

Import validates every setting against current descriptors and reports `Applied`, `IgnoredUnknown`, `Invalid`, `RequiresModule`, `RestartRequired`, `Conflict` before commit.

# Plugin/module settings

A module owns its namespaced SettingIds/schema/migrations. Host stores settings even when module disabled but does not execute validation code from an untrusted missing plugin on config load; schemas needed for safe preservation are host-known from package manifest/last validated metadata where appropriate.

# Config migrations

Configuration schema migrations are independent from document schema. They are pure/deterministic and operate on a copied parsed model. A failed migration retains original file and falls back to last-known-good/defaults with diagnostic.

# Concurrent writes

One application ConfigService serializes persistent writes. Multiple windows mutate the same in-memory settings transactions; they do not independently write files. External manual edit detected during a dirty pending write triggers conflict/reload policy rather than last-writer-wins corruption.

# File watching/manual edits

If config file changes externally:

- re-read/validate as a complete candidate snapshot;
- if current app has no unpersisted settings transaction, atomically publish valid candidate;
- otherwise show/record conflict and require resolution;
- invalid external candidate never replaces current valid snapshot.

# Write durability

Config writes use temp→validate→atomic replace, with small rotating last-known-good backup where policy allows. Successful write only updates persisted-generation marker after final replace.

# Settings observation

Subscribers observe typed `SettingsDelta`/new snapshot generation with changed SettingIds. They do not watch raw files. UI/renderer/module reacts only to declared relevant IDs.

# Performance

Config lookup is snapshot/in-memory and cheap; no filesystem access in render/input hot paths. Resource/theme pack resolution is a separate service; settings usually store selected pack IDs rather than parsing pack data.

# Diagnostics

Expose effective value source (`built-in`, `user`, `workspace`, etc.) in Developer Mode/Preferences advanced view. This makes precedence issues inspectable. Sensitive security store values are never shown.

# Additional gauntlets

- invalid TOML while app is running preserves current snapshot;
- cancel live Appearance preview exactly restores tokens/layout state;
- plugin removed/reinstalled with settings preserved;
- workspace references missing panels and repairs without broken dock tree;
- two windows concurrently edit preferences via transactions;
- external manual config edit conflicts with pending UI change;
- imported preference pack cannot import secrets/grants;
- workspace exported on 2-monitor machine opens safely on 1-monitor machine;
- old config migration failure preserves original;
- generated defaults equal descriptor defaults.