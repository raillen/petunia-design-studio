# 09.24 — Application, Document Session, Multi-Window & Shutdown Lifecycle

# Lifecycle objects

`ApplicationRuntime` owns registries/services; `DocumentSession` owns one open document + history + selection/viewports + jobs; windows/views reference sessions rather than owning documents.

# Startup

Load built-in registries → validate resource packs/config → discover/resolve modules/plugins → initialize platform/renderer/UI → recover sessions/files → show Welcome or restored workspace. A failed optional module cannot block core startup unless declared required.

# Open document

Read/import → validate/migrate → construct DocumentStore → initialize derived services lazily → create DocumentSession → attach one or more views. Opening must not mutate source file.

# Multi-window

One document may have multiple views/windows if supported. Viewport/selection can be per-view or session-shared by explicit policy. History/document dirty state is shared.

# Close document/window

Closing a view does not close document if other views remain. Last-view close triggers unsaved-changes policy. Running jobs either complete, detach safely, cancel or block close according to job kind; no orphaned mutation jobs.

# Dirty state

Document revision vs last-saved revision defines dirty. Autosave does not mark explicit save clean. Derived/cache changes never mark dirty.

# Shutdown

Stop accepting new user work → resolve unsaved docs → cancel/drain jobs → persist workspace/config atomically → plugin/module shutdown with deadlines → renderer/platform teardown. Abrupt crash relies on recovery journal, not graceful shutdown assumptions.

# Reopen/recovery

Recovered document is a new unsaved session pointing back to source/recovery metadata; user decides save destination.

# Tests

Multiple windows same doc, close one, close last with background export, plugin hang during shutdown, config write fail, device loss during close, crash after command before autosave, OS session termination.

# Runtime ownership graph

The application lifecycle must have explicit ownership rather than a mesh of `Arc`s that keeps dead sessions alive accidentally.

Recommended conceptual ownership:

```
ApplicationRuntime
├── ServiceRegistry / CapabilityRegistry
├── ConfigService / ResourceService / PlatformServices
├── JobSystem
├── PluginHost / MCP host adapters
├── RendererDeviceService
└── DocumentSessionManager
    └── DocumentSession (0..N)
        ├── DocumentStore
        ├── DocumentMutator / History
        ├── Derived/Evaluation Session
        ├── SelectionSession(s)
        ├── ViewportSession(s)
        ├── document-owned jobs/watchers
        └── ViewSession(s) referenced by windows
```

Windows/views hold session IDs/weak handles or manager-validated handles. A window cannot keep a fully closed document alive through an accidental strong reference cycle.

# Application lifecycle state machine

```
Cold
→ Bootstrapping
→ Running
→ ShutdownRequested
→ Quiescing
→ PersistingState
→ TearingDown
→ Terminated
```

Boot failure distinguishes `FatalRequiredServiceFailure` from `OptionalCapabilityFailure`. Optional failures are surfaced after startup with diagnostics; they do not strand the app in a half-initialized global state.

# Startup ordering

Canonical startup phases:

1. initialize minimal diagnostics/crash/recovery marker support;
2. resolve platform data/config/cache roots;
3. load/migrate configuration last-known-good snapshot;
4. load built-in resource catalogs + locale/theme fallback;
5. initialize foundational registries/schemas/actions;
6. discover and validate resource packs/modules/plugins without executing document-owned plugin payload;
7. construct JobSystem/platform services;
8. initialize graphics/render device service, with recoverable fallback path where supported;
9. initialize GPUI shell adapter/design system;
10. activate validated optional modules/plugins after permissions/compatibility;
11. scan recovery candidates and requested/open-at-launch files;
12. restore safe workspace/window state;
13. enter `Running`.

No later phase may be required to make an earlier canonical resource/config parse safe. Startup has one clear rollback/diagnostic owner.

# DocumentSession identity

Each open canonical document gets one `DocumentSessionId` distinct from persistent `DocumentId`. Opening the same file twice intentionally may produce either a second view of the existing session or a second independent session only through explicit policy; the app must not accidentally create two writers bound to the same native file.

Default policy: if the same resolved file identity is already open in this process, activate/create another **view** of the existing DocumentSession rather than silently loading a second writer. “Open as Copy/Independent” is explicit and becomes unsaved/unbound unless a safe separate destination is chosen.

# DocumentSession state machine

```
Creating
→ Loading
→ Ready
→ ClosingRequested
→ ClosingBlocked | Closing
→ Closed
```

Transient substates/facts include `Dirty`, `Saving`, `Recovering`, `ReadOnly`, `DegradedResources`, but these should not explode into one giant enum. Keep lifecycle state + orthogonal status flags/operations explicit.

# Open pipeline

Native open:

1. acquire file/grant and identity token;
2. detect native format;
3. parse bounded package;
4. migrate in isolated model;
5. full validate;
6. resolve required/optional capabilities/resources;
7. construct DocumentStore and committed revision;
8. create DocumentSession services/history/evaluation;
9. register linked-resource watchers only after session exists;
10. create initial view/window presentation;
11. expose session as Ready.

Import-open routes through 09.11 staging/import transaction into a **new unsaved document session** unless product flow explicitly converts/imports into current document.

A failed open never creates a partially editable Ready session unless a documented salvage/read-only mode was explicitly chosen.

# Read-only/degraded session

A document may open `ReadOnly` or with degraded optional capabilities when:

- file grant is not writable;
- schema requires newer unsupported required semantics but inspection can be safe;
- repair/salvage mode is active;
- external lock/policy prevents write.

Read-only is an application/session capability state. Commands that mutate report disabled/rejected reason. The core document data is not marked “locked” merely because the OS file is read-only.

# ViewSession vs DocumentSession

A `ViewSession` owns presentation/session state for one document view:

- viewport transform/current Surface focus;
- view-local canvas display options where designed;
- window/tab relationship;
- active tool context if policy is view-local;
- temporary focus/HUD state.

The document/history/dirty state is shared by all views.

# Selection ownership decision

V1 recommendation: **selection is per ViewSession by default**, because two windows may inspect different areas/objects without fighting each other. Commands receive explicit target IDs/selection snapshot from their invoking view. Some application-level operations may synchronize/replace selection intentionally through a semantic request.

Presentation APIs always identify the `ViewSessionId` whose selection is being queried. Never rely on one process-global `current_selection` singleton.

# Active tool ownership

Active tool is also view/workspace-session state, not canonical document data. Switching Design↔Photo may change the active/default tool for that view according to workspace policy, but cannot alter another detached view unexpectedly unless the user chose synchronized views.

# Multi-view mutation behavior

All views observe the same committed ChangeSets and revision. If View A commits a mutation affecting View B's selected/deleted objects, View B reconciles selection through stable-ID rules:

- deleted selected ID removed;
- surviving IDs retained;
- optional selection-recovery hint may choose replacement/parent;
- viewport is not automatically moved unless action semantics require.

# Save operation ownership

Only DocumentSession coordinates native Save/Save As for its file binding. Multiple windows cannot concurrently run conflicting Save operations for the same session.

Save requests serialize through a session save coordinator:

- if equivalent save already active, UI can observe/join it;
- Save As with different destination cannot race the active save silently;
- successful save updates `last_saved_revision`, file identity and binding once;
- edit commands may continue during snapshot-bound save; new edits keep document dirty after save completes if revision advanced.

Example: save captures revision 20, user edits to 22, save(20) succeeds → `last_saved_revision=20`, current=22, document remains dirty.

# Dirty-state model

Track:

```
current_committed_revision
last_explicitly_saved_revision
last_autosaved_revision
last_exported_revision(s) optional metadata
```

Dirty = current committed revision differs from last explicitly saved semantic state/binding. Undoing exactly back to a saved state may clear dirty only if state identity/hash/revision lineage can prove equivalence; do not clear merely because undo stack index resembles an older position after branching.

A robust implementation may maintain a saved-state semantic fingerprint/history marker alongside revision.

# Close-view semantics

Closing one tab/window destroys only that ViewSession. If at least one other ViewSession references the document, no unsaved-document prompt is shown solely because this view closed.

If floating panels are application/workspace-global, they rebind to the newly active view/session or enter no-document state through explicit presentation logic; they do not own the document.

# Last-view close protocol

When closing the last user-visible view:

1. mark session `ClosingRequested` and stop creating new view-owned interactions;
2. inspect dirty/read-only/save state;
3. inspect document-owned jobs/transactions;
4. if dirty, request Save / Don't Save / Cancel through semantic dialog flow;
5. resolve active edit transaction — generally commit/cancel via owning tool flow before final close;
6. resolve jobs according to JobKind ownership (cancel mutation/derived; optionally detach snapshot-bound export);
7. stop linked watchers/subscriptions;
8. flush recovery metadata/session settings as required;
9. unregister session from manager;
10. drop DocumentStore/derived resources;
11. mark Closed.

`Cancel` at any user decision returns the session to Ready and restores ability to interact.

# Background export on close

An export operating solely on an immutable captured snapshot may be **reparented from document-owned to application/batch-owned** and continue after the document view/session closes, provided:

- it no longer references DocumentSession mutable/session objects;
- destination grant remains valid;
- UI/task center can still report it;
- user is informed if closing the app would cancel it.

A mutation/import targeting the session cannot detach this way.

# Active transaction on close

Interactive drag/text/brush transaction is owned by a view/tool. Last-view close first asks the tool/session to resolve it. Normal close should commit the user's completed current gesture if the tool has already received its commit event; an in-progress gesture cancelled by window close/Esc restores pre-transaction state. Never serialize a half-preview state.

# Linked-resource watchers

Watchers are document-session owned. On external change they create a pending issue/event and only apply canonical update through explicit policy/Command. Closing the session cancels watcher callbacks before DocumentStore teardown.

# External file deletion/rename

If the bound native file disappears after open, the in-memory document remains valid and becomes `FileBindingMissing`. Save offers recreate at prior destination/Save As according to platform grant capability. It is not silently treated as a clean untitled document.

# OS sleep/resume

ApplicationRuntime receives platform lifecycle events. Before suspend, best-effort recovery checkpoint may be scheduled if dirty and time permits; never block indefinitely. Resume revalidates GPU/display/font/watch services and marks derived device-bound state dirty without document mutation.

# Shutdown request sources

Normalize user Quit, last-window quit policy, OS session end and updater restart into a shutdown coordinator with source/deadline metadata. OS-forced deadlines may skip normal prompts and rely on recovery/autosave rather than risking indefinite block.

# Application shutdown protocol

1. transition to `ShutdownRequested`, stop accepting new ordinary user/plugin/MCP mutation sessions;
2. enumerate open DocumentSessions and resolve dirty documents in a deterministic visible order where interaction is possible;
3. resolve/await in-progress saves needed for user choice;
4. cancel or allow explicitly approved snapshot-bound foreground jobs according to exit policy; app cannot terminate while a writer still owns destination unless result is abandoned safely;
5. disable MCP listener/new external requests;
6. quiesce plugins/modules with bounded deadlines and rollback abnormal transactions;
7. persist workspace/config/window state atomically;
8. flush diagnostics/recovery indexes;
9. tear down UI windows/render surfaces;
10. tear down GPU/device/platform services;
11. stop executors/job pools;
12. mark clean-shutdown marker and terminate.

# Hung module/plugin during shutdown

Every extension/module has a bounded shutdown deadline. In-process Lua plugin that fails to quiesce is interrupted/destroyed after host rolls back active plugin mutation and revokes capabilities. Optional Wasmtime instance is interrupted according to its runtime controls. Core shutdown proceeds; one plugin cannot hold the application forever.

# Clean-shutdown marker

Maintain a minimal durable marker/session index indicating whether previous run reached clean shutdown. On next start, absence/dirty marker triggers recovery candidate scan and crash/support diagnostics. This marker contains no creative content.

# Recovery vs session restore

Do not confuse:

- **recovery:** protects unsaved committed document state after crash;
- **session restore:** remembers which documents/windows/workspaces were open.

Session restore attempts to reopen explicit saved files and presents recovery candidates separately when newer. It does not silently replace saved documents with recovered state.

# Session restore policy

Restore list contains file identity/grants where platform persistence allows, view/window workspace metadata and active view info. Failure to reopen one file does not block others. Missing file becomes a recoverable recent/session issue.

# Service dependency teardown

Services cannot assume arbitrary destruction order from Rust drop. ApplicationRuntime explicitly tears down in dependency-safe order. Background callbacks use owner tokens/generations so late callbacks become no-ops rather than dereference torn-down services.

# Memory/resource release

Closing a DocumentSession should release document-specific canonical resources, derived caches, GPU scene resources, watchers and undo spill references once no snapshot-bound external job owns its immutable copies. Developer diagnostics include leaked owner/job/resource counts after close in gauntlet builds.

# Lifecycle diagnostics

Every transition emits structured state change with reason/source/correlation ID. Invalid transition is a debug/test failure and release diagnostic, not silently ignored.

# Additional lifecycle gauntlets

- same file opened from second window resolves to existing session rather than second writer;
- two views with independent selections editing same document;
- save revision N while user edits to N+2 remains dirty after save;
- close one of two views without prompt;
- cancel last-view close returns exact Ready state;
- background snapshot export continues after session closes with no dangling callbacks;
- linked watcher event races close safely;
- file deleted externally then Save/Save As recovery;
- plugin hangs during shutdown and cannot block termination indefinitely;
- OS forced session end with dirty docs preserves latest committed recovery state;
- crash marker/session restore distinguishes recovery from ordinary reopen;
- repeated open/close cycle leaves zero leaked document-owned jobs/watchers/GPU resources.