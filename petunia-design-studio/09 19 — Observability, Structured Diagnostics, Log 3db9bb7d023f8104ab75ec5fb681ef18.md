# 09.19 — Observability, Structured Diagnostics, Logging, Profiling & Crash Bundles

# Principle

A complex creative tool must explain *why* it is slow or failing without requiring a debugger. Observability is designed into services.

# Structured diagnostics

`Diagnostic` contains stable code, severity, subsystem/module, TextId, typed parameters, related object/resource IDs, source error chain and suggested machine-readable actions where applicable.

# Logging/tracing

Use `tracing`-style structured spans/events across actions, commands, jobs, evaluation, render, import/export and plugins. Release default avoids verbose document content and PII.

# Correlation

Action/Command/Job/Export/Plugin invocation IDs correlate layers. A user-reported failed export can be traced from UI action through adapter phases.

# Developer diagnostics

Panels/window for FPS/frame time, GPU memory, tile cache, evaluation dirty graph, job queue, render scene stats, text/font resolution, color transforms, resource links, plugin state and recent diagnostics.

# Performance marks

Record p50/p95 for interactive actions, scene extraction, layout, frame rendering, boolean operations and large-tree UI where benchmarkable. Budgets live with gauntlet docs.

# Crash reporting

Local crash bundle: app/build/platform info, panic/backtrace when available, recent redacted structured events, enabled module/plugin versions, GPU adapter/driver and recovery/autosave location metadata. User explicitly chooses whether to share externally.

# Diagnostics export

Support “Copy Diagnostic”/“Save Support Bundle” with configurable redaction; never package the creative document unless user explicitly selects it.

# Error codes

Stable diagnostic codes support docs/search/automation even when localized message changes.

# Tests

Redaction tests, plugin error isolation, tracing correlation, log rotation/storage bounds, support bundle without private content, crash during active autosave/export.

# Diagnostic taxonomy

Stable diagnostics use a code namespace and typed severity:

```
Severity = Info | Warning | Error | Fatal
DiagnosticCode examples:
AUB-DOC-001
AUB-IMPORT-SVG-014
AUB-COLOR-ICC-007
AUB-PLUGIN-PERM-003
```

Codes are stable compatibility/search identifiers. Localized message wording may evolve without changing code. Plugin diagnostics use plugin namespace and cannot claim `AUB-*` built-in codes.

# Diagnostic payload

Canonical `Diagnostic` should include:

- code;
- severity;
- subsystem/module/provider ID;
- `TextId` + typed parameters;
- correlation/trace ID;
- document/session ID when applicable;
- related semantic object/resource IDs;
- source error category/chain for developer details;
- structured recovery actions (`Retry`, `ChooseProfile`, `Relink`, `DisablePlugin`, etc.);
- privacy classification/redaction hints;
- timestamp only as operational metadata.

Diagnostics are data. UI decides banner/dialog/panel/toast according to 08.13; services do not instantiate GUI alerts.

# Event/log levels

Use structured tracing levels with explicit intended audience:

- `TRACE` — high-volume developer internals, disabled in release default;
- `DEBUG` — subsystem state useful in developer builds;
- `INFO` — lifecycle/major operations without user content;
- `WARN` — recoverable abnormal behavior/degradation;
- `ERROR` — failed operation requiring diagnostics;
- crash/fatal path — minimal guaranteed flush metadata.

Do not log expected user cancellation as `ERROR`.

# Privacy classification

Every structured field type is classified conceptually:

1. **Safe operational** — stable action IDs, durations, revision counts, crate/module versions;
2. **Potentially identifying** — file names/paths, plugin IDs from private source, machine/user account info;
3. **Document content** — text, object names, image pixels, data-merge records, clipboard;
4. **Secret** — tokens, credentials, environment secrets, signed URLs.

Release logging defaults to class 1. Classes 2–4 require explicit redaction/diagnostic opt-in and never appear accidentally through generic `Debug` formatting of domain structs.

# Redaction policy

Central redaction service handles path/string/resource fields. Preferred support-bundle representations:

- paths → basename optionally + stable session hash, parent path redacted;
- document/object IDs → allowed if random/nonsecret but can be hashed in shared bundle mode;
- URLs → scheme/host only unless user opts in;
- user text/data → omitted by default;
- credentials/query secrets → always removed.

`#[derive(Debug)]` output is not an acceptable privacy policy.

# Trace hierarchy

Representative span tree:

```
Action(aubrieta.action.export)
└── Command/ExportRequest
    ├── Preflight
    ├── Evaluation snapshot
    └── Job(export)
        ├── Encode
        ├── Validate
        └── Atomic commit
```

Plugin/MCP invocations add source/provider span parents. Correlation IDs propagate across async jobs and back to diagnostics/results.

# Metrics model

Metrics are local diagnostics by default, not remote telemetry. Track counters/gauges/histograms such as:

- frame time p50/p95/p99;
- command latency;
- job queue wait/run;
- cache hit/miss/bytes;
- raster CPU/GPU memory;
- export/import phase duration;
- text layout/shaping duration;
- plugin host calls/quota failures;
- MCP request counts/result categories;
- recovery/autosave duration/failure;
- diagnostic counts by code/severity.

Labels must have bounded cardinality. Never use ObjectId/file path/user text as metric label.

# Performance budget ownership

Each subsystem documents its representative benchmarks/budgets. Observability supplies measurements but does not define success globally. CI stores benchmark baselines and detects material regressions with noise-aware policy; one microbenchmark result cannot override end-to-end interaction gauntlets.

# Log storage

Local log writer has bounded size/age/rotation. A runaway plugin/parser cannot fill disk indefinitely. Retention policy is configurable and exposed in Preferences/Support diagnostics where useful.

On write failure/no-space, logging degrades gracefully and emits at most bounded fallback indication; failure to write a log never recursively floods logging.

# Support bundle manifest

A support bundle is a versioned archive containing only selected/redacted artifacts:

```
support-manifest.json
app-build.json
platform.json
diagnostics.jsonl
recent-traces.jsonl or structured equivalent
modules-plugins.json
gpu-display.json
config-summary-redacted.json
recovery-metadata-summary.json
```

Creative documents, images, font files, ICC payloads, plugin storage and full user configs are excluded by default.

User sees a checklist/summary of what will be included before saving/sharing support bundle.

# Crash bundle critical path

Crash handler must avoid allocations/locks/complex subsystem callbacks where unsafe. Maintain a small ring buffer or preformatted recent-event store suitable for best-effort crash capture. Full support bundle enrichment can happen on next launch from safe recovery context.

# Crash fingerprint

Generate a local crash fingerprint from app/build + top frames/panic location + subsystem context where possible. It helps group repeated crashes without uploading automatically.

# Panic hooks

Panic hook records minimal redacted metadata and triggers recovery/crash marker. It must not attempt canonical Save of possibly inconsistent in-memory state. Recovery relies on last known committed autosave/checkpoint according to 09.10.

# GPU diagnostics

Capture adapter/backend/driver identifiers, limits/features needed for renderer debugging without assuming they are stable identity. Device-loss reason/count and recent allocation pressure are valuable. Do not dump texture contents/shaders containing document-derived data.

# Plugin diagnostics

Per plugin expose:

- package/runtime version;
- lifecycle state;
- granted/requested capabilities;
- recent invocation result codes;
- CPU/memory/host-call quota stats;
- active jobs/contributions;
- last trap/error code with redacted stack/script location.

Ordinary plugin users can copy a plugin-specific diagnostic without exposing unrelated document content.

# MCP diagnostics

Track transport/session identity class, permission profile, request method/schema version, revision precondition, duration/result code and correlation ID. Arguments that contain document data/paths are not logged by default.

# Diagnostic action registry

Recovery actions referenced by `Diagnostic` are semantic IDs resolved by application layer, e.g. `aubrieta.recovery.choose_profile`. A diagnostic never embeds an executable closure/raw URL as recovery behavior.

# Diagnostics retention by scope

- ephemeral UI diagnostics may expire after session;
- document issue diagnostics are recalculated from document state and not blindly persisted;
- support logs retain bounded operational history;
- crash markers persist until acknowledged/recovery processed;
- security/audit-sensitive plugin grants follow settings/security persistence, not generic logs.

# User-facing Developer Mode

Developer mode can enable richer traces/inspectors but displays a clear indicator and still preserves secret redaction. It can expose:

- live span/job timeline;
- Action/Command inspector;
- evaluation/cache graph;
- render pass/memory stats;
- UI focus/accessibility tree;
- token/resource resolution;
- plugin/MCP calls with sanitized args;
- recent diagnostics browser/filter.

# Exportable trace format

Prefer machine-readable structured JSON/JSONL or an established trace format adapter so code agents/tools can analyze without scraping localized text. The on-disk format is versioned and documented if considered a support contract.

# Failure-injection observability

Gauntlet tests assert not only recovery but **diagnostic quality**: stable code, affected ID, safe message parameters, recovery action and correlation. Generic `operation failed` without machine-readable reason is a test failure for known failure classes.

# Additional tests

- high-volume logging cannot exceed rotation budget;
- document strings/paths/secrets absent from default logs/support bundle;
- same DiagnosticCode renders correctly in en-US and pt-BR;
- correlation survives Action→Command→Job→Exporter async chain;
- plugin/MCP redaction under malicious argument values;
- panic during autosave uses previous recovery checkpoint and does not attempt unsafe Save;
- support bundle generation itself handles disk full/cancel safely;
- Developer Mode off removes high-volume tracing overhead within budget.