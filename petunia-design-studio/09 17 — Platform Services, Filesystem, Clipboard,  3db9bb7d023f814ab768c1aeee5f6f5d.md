# 09.17 — Platform Services, Filesystem, Clipboard, Dialogs, Fonts, Pen & OS Integration

# Boundary

Operating-system integration is behind `petunia_design_platform` service traits/adapters. Domain crates do not call platform APIs directly.

# Service groups

- paths/directories and atomic filesystem operations;
- native open/save/folder dialogs;
- clipboard text/image/custom document fragments;
- drag/drop and promised files where supported;
- font enumeration/change notifications;
- monitor/HiDPI/color-profile discovery;
- pointer/tablet/pen capabilities;
- native menus/window controls/notifications where Slint exposes them;
- URL/help opening;
- power/session lifecycle and safe shutdown hooks.

# Path semantics

Internally use normalized path abstractions without assuming UTF-8 display strings. UI presentation of path is separate. Symlink/canonicalization/security decisions belong to platform/file services.

# Clipboard

Define Petunia rich fragment MIME/type plus standard SVG/PNG/text fallbacks. Paste selects richest trusted supported representation according to policy. Clipboard data is hostile input.

# File watching

Linked asset/config/resource-pack watchers debounce/coalesce changes and compare fingerprints. Watch events are hints; always re-read/validate.

# Color/monitor

Monitor profile change causes display transform invalidation, not document mutation.

# Pen

Normalize pressure/tilt/rotation/buttons into Petunia pointer events with device capability metadata; no tool depends on one OS tablet API.

# Tests

Mock platform services for headless unit tests; integration matrix per Windows/macOS/Linux for Unicode paths, DPI, clipboard, IME/pen, multi-monitor profile changes and file dialog behavior.

# Platform capability interfaces

`petunia_design_platform` should expose small typed ports rather than one giant `Platform` object. Recommended logical services:

- `FileSystemService`;
- `FileDialogService`;
- `ClipboardService`;
- `DragDropService`;
- `FontSystemService`;
- `DisplayService`;
- `InputDeviceService`;
- `NotificationService`;
- `UrlLauncherService`;
- `PowerSessionService`;
- `ProcessEnvironmentService` only for narrowly approved read-only metadata.

Feature/domain crates depend on the narrowest required port. The Slint adapter may satisfy several ports internally, but that does not collapse their semantic contracts.

# Filesystem path model

Use Rust/OS-native path representation (`Path`/owned equivalent) internally. A path has three separate representations:

1. native path used for filesystem calls;
2. normalized identity/comparison token produced by platform service where needed;
3. lossy/localized display string for UI only.

Never serialize a lossy display string as the only canonical linked-resource path.

# File identity

For external-modification/conflict detection, platform service can provide an opaque `FileIdentityToken` derived from available metadata such as device/file ID, size, modified time and/or content fingerprint. Higher layers compare tokens without assuming inode semantics exist identically on every OS.

# Atomic replacement API

Expose a high-level `atomic_replace(temp, destination, durability_policy)` contract used by native save/export/config writes. The platform adapter owns Windows/macOS/Linux differences, backup/rename semantics and best-effort fsync behavior.

Callers must not hand-roll `remove(destination); rename(temp)` sequences that create a no-file window.

# Temporary files/directories

Provide scoped temp resources with RAII/cleanup semantics and explicit persistence/commit operation. Security-sensitive temp files are created with safe permissions and unpredictable names. Temp roots for recovery, export staging, plugin staging and undo spill are separate logical namespaces.

# Scoped file grants

Plugin/MCP/user-mediated file access uses opaque `FileGrant`/`DirectoryGrant` handles produced by dialog/policy services. A grant declares allowed read/write/create scope and lifetime. External actors should prefer grants over arbitrary string paths.

# Symlink/reparse handling

Every operation that needs containment (`resource pack`, `plugin package`, `scoped grant`) defines whether symlinks/reparse points are followed. Containment validation must operate on platform-safe resolved semantics and guard TOCTOU where practical. Archive extraction never creates/follows entries that escape staging root.

# File dialogs

Dialog request is a typed DTO:

```
FileDialogRequest
- kind: OpenFile/OpenFiles/SaveFile/ChooseFolder
- semantic title TextId
- format filters with MIME/extensions
- suggested name/extension
- initial location policy
- grant scope requested
```

Result contains native path/grant + chosen filter/format identity. Domain code does not parse a localized filter label to determine format.

Cancellation is a normal result, not an error.

# Save dialog extension behavior

If the user selects the Petunia Design Studio native format, platform/file workflow applies `.PTND` as the canonical suffix. Legacy `.aubrieta`/`.aubri` are migration-open inputs only and are not ordinary Save As targets. Format identity comes from the selected filter/adapter plus package metadata, not blindly from typed suffix. Suffix/format mismatch receives deterministic validation instead of silently writing the wrong format.

# Clipboard format negotiation

Petunia writes multiple flavors when feasible:

1. private Petunia semantic fragment with version/media type;
2. standard SVG/PDF/vector flavor appropriate to platform;
3. PNG/raster fallback;
4. plain text when meaningful.

Paste enumerates available flavors and chooses richest **safe** supported representation under policy. Private fragment is validated/versioned exactly like other untrusted serialized input.

# Clipboard privacy

Clipboard read happens only in direct user operation or explicitly granted automation/plugin capability. Petunia does not continuously poll clipboard contents. Diagnostics never log clipboard payload by default.

# Drag and drop

Normalize drag payload into semantic items: files/grants, text, URLs, Petunia fragment, internal object drag token. Internal drags use process/session-scoped opaque IDs and never expose raw pointers.

External drop data is hostile. URLs do not auto-fetch network content; files go through importer/security limits.

# File watching

Watcher API produces `ChangedHint { watched_resource_id/path_token, event_kind, sequence }`. Events may be duplicated, coalesced, reordered or absent on some filesystems; consumers always stat/read/fingerprint to determine real change.

Watcher teardown is owner-scoped so closing a document/resource pack/plugin leaves no callbacks into dead state.

# Font system service

Expose immutable font-index snapshots/deltas with semantic face metadata and stable session fingerprints. Enumeration/scanning happens off UI thread. Font files are hostile inputs and parsed through reviewed typography/font stack.

System font change invalidates font resolution/layout derived state as needed but never rewrites requested font identity in document.

# Display/monitor service

Per display expose stable session ID, logical/physical geometry, scale factor, refresh information where reliable, HDR/color capabilities if later used and resolved display ICC profile/reference.

Moving a window across displays updates GUI scale/render/display transform through session state. Document does not become dirty.

# Pen/pointer normalized event

Conceptual event:

```
PointerSample
- device_id/session_id
- device_kind: mouse/pen/touch/trackpad-derived
- phase/buttons
- logical window position
- pressure? normalized 0..1
- tilt_x/tilt_y?
- azimuth/rotation?
- tangential_pressure?
- timestamp
- coalesced/predicted metadata if platform exposes it
```

Unavailable capabilities are `None`, not invented defaults presented as real hardware data. Brush engine defines fallback curves.

# Predicted/coalesced input

Platform predicted pointer events may improve preview but must not enter committed deterministic stroke data unless brush pipeline explicitly resolves/corrects them. Commit uses normalized accepted sample stream; prediction is derived visualization.

# IME/platform text input

Platform/Slint text input adapter provides composition lifecycle, candidate anchor placement and committed text to `petunia_design_text`. IME APIs remain outside document crate. Global shortcuts are suppressed while composition contract requires it.

# Native menus/window controls

Where Slint/platform integration provides native menu/titlebar conventions, UI adapter maps semantic Actions to them. Platform menu item labels/shortcuts remain TextId/Action metadata; native callbacks invoke Action IDs rather than custom business closures.

# Notifications

OS notifications are opt-in/useful only for long background completion when application/window context justifies them. Notification content is localized semantic text; clicking resolves to a safe application route. Plugins cannot post arbitrary OS notifications without explicit capability.

# URL launching

Only explicit user/plugin-capability action can open an external URL. Validate supported scheme (`https` baseline; documented safe schemes) and display/confirm suspicious nonstandard schemes where necessary. Help topics resolve through HelpTopicId; feature code does not hardcode URLs.

# Power/session lifecycle

Service reports suspend/resume, session end/logoff/shutdown request where OS supports. Petunia uses this to:

- request recovery/autosave checkpoint when feasible;
- quiesce GPU/IO appropriately;
- restore display/device resources after resume;
- never block OS shutdown indefinitely.

# OS permission failures

Platform service maps raw OS errors into typed categories (`NotFound`, `PermissionDenied`, `ReadOnly`, `AlreadyExists`, `NoSpace`, `PathTooLong`, `Busy`, `Unsupported`, `IoFailure`) while retaining source error for diagnostics. User-facing recovery is chosen by higher-level workflow.

# Headless/mock contract

Every platform port has deterministic mock implementation sufficient for core/application tests. Tests can inject:

- file contents/errors/external modification;
- dialog selection/cancel;
- clipboard flavors;
- display profile/scale changes;
- watcher duplicate/lost events;
- pen capability streams;
- suspend/resume/shutdown.

No domain test requires a real desktop session.

# Cross-platform conformance matrix

Release-supported OS matrix must explicitly test:

- Unicode/non-UTF8-representable native paths where platform allows;
- very long paths;
- read-only/network/removable locations where practical;
- clipboard rich-format roundtrip;
- file dialogs and extension/filter mapping;
- drag/drop multi-file;
- font install/remove notifications;
- mixed-DPI/multi-monitor transitions;
- pen pressure/tilt when hardware/test harness available;
- suspend/resume and safe shutdown;
- atomic replacement conflict/failure behavior.

# Code-agent rule

An agent needing OS functionality must first search `petunia_design_platform` for an existing port. Direct `std::fs`/Win32/AppKit/GTK/desktop API calls inside feature/domain modules are architecture violations except in the platform adapter implementation itself or tightly scoped build tooling.