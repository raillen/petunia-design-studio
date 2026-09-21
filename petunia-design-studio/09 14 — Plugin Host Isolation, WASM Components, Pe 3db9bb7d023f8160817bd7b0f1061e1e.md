# 09.14 — Plugin Host Isolation, WASM Components, Permissions & Versioning

# Plugin host model

`aubrieta_plugin_host` owns the **runtime-neutral security and capability boundary**. The semantic Plugin SDK is defined independently from the scripting/runtime implementation so choosing Lua/mlua, Python, JavaScript or a WASM component tier cannot redefine document, Action/Command, property, permission or UI semantics.

Plugins never receive Rust ABI pointers, mutable document references, raw GPUI objects, GPU resources or unrestricted process memory.

# Runtime tiers — decision status

**Resolved V1 decision:** the approachable primary scripting runtime is **Lua 5.5 through `mlua`**. It implements the runtime-neutral contracts defined in 09.28 and must not redefine document, Action/Command, property, permission or UI semantics. Python and JavaScript remain external automation/client-SDK options and research/fallback candidates, not peer embedded V1 runtimes.

**V1_REQUIRED architecture seam:** the semantic Plugin SDK/capability broker must remain compatible with a stronger isolated component adapter. **Wasmtime + WASI Component Model is the preferred `POST_V1_CANDIDATE` high-isolation execution tier** unless a named V1 milestone explicitly promotes it after equivalent semantic/security conformance. It is not the ordinary scripting runtime and cannot introduce a second semantic API.

The architecture therefore permits two tiers only in this form:

1. **Lua/mlua V1 scripting tier** — small, approachable, one Lua state per plugin by default, host-brokered capabilities;
2. **WASM/Wasmtime high-isolation tier — POST_V1_CANDIDATE by default** — advanced components requiring stronger isolation or a cross-language component ABI.

Both tiers consume the same semantic Plugin SDK/capability broker and contribution schemas. Any feature available to one tier but not the other must be classified as a runtime capability difference, never implemented as parallel business logic.

# Manifest

TOML manifest: namespaced plugin ID, semantic version, Aubrieta compatibility range, Plugin SDK/API version, selected runtime/runtime requirement, requested capabilities, contributions, bundled resources, dependencies/conflicts, author/license/source/signature metadata. WASM components additionally declare component/world version where applicable.

# Interfaces

Define stable coarse-grained semantic interfaces first. Runtime bindings adapt those interfaces to the chosen scripting language. WASM/component implementations use versioned WIT worlds for the same semantic operations. Initial surfaces may expose document read snapshots, Command/Action requests, typed property schemas, effect/import/export/data-source contracts, jobs and resource registration. UI-specific plugin surfaces remain declarative and narrower.

# Capability permissions

Examples: `document.read`, `document.write`, `filesystem.read:scoped`, `filesystem.write:scoped`, `network:https`, `clipboard.read/write`, `ui.panel`, `register.importer`, `register.exporter`, `background.job`.

Default is deny. User grants are explicit, inspectable and revocable. A plugin cannot escalate by registering another contribution.

# Versioning

Version independently: Plugin SDK semantic API, package/manifest schema, declarative UI schema and any runtime binding. Backward-compatible additions do not break old plugins. Breaking changes require a new major API version; WASM/WIT breaking interfaces create a new world/version where that tier is used. Runtime choice or runtime-library upgrades must not silently change the semantic API. Compatibility is checked before loading/instantiation.

# Resource quotas

Memory, CPU/time/instruction budget where the runtime supports it, open files, network request limits, produced output size, UI update frequency and concurrent jobs are bounded. Wasmtime may use fuel/epoch interruption; scripting runtimes need equivalent host interruption/watchdog strategy before being considered safe for untrusted distribution. Long-running plugin work is cancelable and may not hold a canonical document transaction open indefinitely.

# Document-owned extension data

Plugin-defined document payloads are namespaced/versioned and preserved opaquely if plugin unavailable. Loader must never execute plugin code merely to inspect a document.

# UI contributions

**Resolved V1 policy:** plugin UI is declarative and semantic. Plugins contribute `PanelSchema`/`FormSchema`/property descriptors, actions, icons, strings and data models; the host renders them with Aubrieta-native tokenized controls. Arbitrary native GPUI object injection is permanently forbidden across the plugin boundary.

For genuinely complex plugin interfaces, a sandboxed embedded surface/WebView tier is **POST_V1_CANDIDATE** and requires its own accepted UI/security ADR before implementation. It would use a message bridge with explicit lifecycle, permissions and resource budgets. That tier remains isolated from document pointers, GPU internals and GPUI native objects. The declarative native tier remains the default because it automatically inherits theme, localization, accessibility, keyboard and design-system behavior.

# Failure isolation

Plugin crash/trap disables that instance/contribution, reports diagnostics and preserves document data. Core remains usable.

# Distribution/trust

**V1_REQUIRED distribution baseline:** local/manual install with validation, permissions, origin and license visibility. A curated/remote marketplace is **POST_V1_CANDIDATE** and requires its own signing, trust, moderation and update-distribution contracts. Track hashes/signatures when the accepted distribution infrastructure provides them.

# Tests

Malicious manifests, denied capabilities, traps/timeouts, oversized memory/output, incompatible version, plugin missing on reopen, opaque data roundtrip, plugin disable with active job, duplicate contributions.

# Lua/mlua V1 host profile

The accepted V1 Lua tier is intentionally narrower than a general desktop Lua installation.

Host defaults:

- vendored Lua 5.5 runtime; no external Lua installation required;
- one Lua state per plugin instance by default;
- `mlua` safe constructor/library selection; unrestricted `io`, `os`, `debug`, native C module loading and ambient package search are unavailable to ordinary plugins;
- host-controlled `require` resolves bundled pure-Lua modules + Aubrieta SDK modules only;
- per-plugin memory limit/accounting;
- instruction/time interruption/watchdog;
- no FFI/native module escape hatch for untrusted plugins;
- filesystem/network/clipboard/process/environment access only through explicit Aubrieta capabilities;
- durable plugin state lives in namespaced host storage, not trusted VM globals;
- unload/hot reload destroys the VM after jobs/transactions/contributions quiesce.

# Lua host object rule

Rust objects exposed to Lua are opaque host handles/DTOs with bounded lifetime and permission checks. Do not expose raw pointers, `Arc<Mutex<DocumentStore>>`, GPUI entities or GPU handles through userdata.

Preferred API style is coarse-grained semantic calls:

```
aubrieta.actions.invoke(...)
aubrieta.document.query(...)
aubrieta.document.transaction(...)
aubrieta.ui.register_panel(schema)
aubrieta.jobs.spawn(...)
aubrieta.storage.get/set(...)
```

Avoid high-frequency host↔Lua calls per pixel, path vertex, glyph or frame. Batch/query APIs are required for scale.

# Permission grant lifecycle

Permission state is host-owned and keyed by plugin identity/version/signing/origin policy. Install/update flow compares requested capability set with previously granted set.

- unchanged or reduced permissions may preserve grants according to policy;
- any new/increased sensitive capability requires renewed explicit consent;
- revocation takes effect at the broker and cancels/blocks future privileged operations;
- revoking a capability cannot be bypassed by cached Lua function references.

# Filesystem grants

Ordinary plugins do not receive arbitrary paths as ambient authority. The broker returns scoped file/directory handles/grants created by user choice or manifest-approved application storage. A handle contains the authority; converting it back into unrestricted path traversal is forbidden.

# Network grants

Network capability is explicit and separately revocable. V1 should prefer HTTPS-only outbound requests with host-enforced destination/size/time limits. Credentials/cookies are never inherited from the application/environment. A credential-broker API is **POST_V1_CANDIDATE** and would require an explicit security/consent ADR before any plugin can receive managed credentials.

# Document mutation

A plugin mutation is either:

- a normal semantic Action/Command request; or
- a scoped host transaction using the same validation/undo/revision rules.

Lua code never mutates document structs directly. If the plugin errors, times out, is cancelled or exceeds quota before commit, the host rolls back the active transaction exactly.

# Plugin lifecycle state machine

```
Discovered
 → ManifestValidated
 → CompatibilityChecked
 → PermissionResolved
 → RuntimeCreated
 → ContributionsRegistered
 → Active
 → Suspending
 → Quiesced
 → Unloaded
```

Failure at any stage rolls back contributions/resources registered by that instance. `Active → Disabled/Error` follows the same quiesce/unregister cleanup path.

# Runtime teardown

Unload sequence extends 09.1:

1. reject new plugin calls;
2. mark contributions unavailable for discovery;
3. revoke/cancel privileged pending operations;
4. cancel/quiesce plugin jobs;
5. commit or rollback owned document transaction according to explicit state — default rollback on abnormal termination;
6. detach UI/actions/importers/exporters/data sources;
7. flush bounded host-owned plugin storage;
8. drop host handles/userdata;
9. destroy Lua VM / Wasmtime instance;
10. verify no listeners/jobs/contributions remain.

# Declarative UI schema security

Plugin `PanelSchema`/`FormSchema` supports only host-defined control primitives and typed properties/events. Schema cannot include arbitrary GPUI code, script snippets in style fields, external URLs that auto-load, raw shader code or unrestricted HTML.

All visible text/icons are namespaced plugin resources and go through the same token/localization/accessibility system. Host may reject layouts that exceed depth/control/update-frequency limits.

# High-isolation WASM tier

Wasmtime components receive capability-oriented WIT interfaces mirroring the semantic SDK. WASI capabilities are not enabled wholesale. Preopened directories/network/socket/process access are absent unless brokered by an Aubrieta-specific permission contract.

Component resource limits include memory, table/instance count, fuel/epoch interruption where appropriate, output sizes and concurrent host calls/jobs.

# Version compatibility

Plugin compatibility is checked before runtime creation against:

- manifest schema version;
- semantic Plugin SDK major/minor range;
- selected runtime binding version;
- declarative UI schema version;
- contribution/property schema versions used;
- optional WASM world/interface version.

A plugin incompatible with the current host remains installed but disabled with actionable diagnostics; Aubrieta does not attempt best-effort invocation of a mismatched ABI.

# Plugin update atomicity

Updating a plugin stages and validates the new package separately. The active old version remains available until the new package passes manifest/security/compatibility validation and any permission change is resolved. If activation fails, roll back to old package/state rather than leaving half-updated contributions.

# Quota response

Quota violations are structured outcomes (`MemoryLimit`, `CpuBudget`, `HostCallRate`, `OutputLimit`, `JobLimit`, etc.). The host terminates/cancels the offending invocation/instance according to severity, rolls back open mutations and records diagnostics. Quota failure is not reported as a generic script error.

# Plugin package safety

Plugin package extraction obeys the same archive path/count/decompression protections as native/resource packs. Package data never writes outside the plugin install/staging root. Native executable libraries in ordinary Lua plugins are rejected by default.

# Required V1 plugin gauntlets

- 100 trivial Lua plugins instantiated/disabled/reloaded with memory/startup telemetry;
- infinite loop interrupted within accepted latency;
- memory-growth plugin stopped at quota;
- denied file/network/clipboard attempts with no ambient fallback;
- permission revocation while plugin holds cached API objects;
- exception/timeout during document transaction restores exact state;
- disable/unload while background job and panel are active;
- update requesting new permission requires renewed consent;
- corrupt/malicious manifest/archive/native module attempt rejected;
- missing plugin on reopen preserves opaque document data;
- same semantic test plugin implemented in Lua and WASM tier yields equivalent document Action/Command result where both support the capability.