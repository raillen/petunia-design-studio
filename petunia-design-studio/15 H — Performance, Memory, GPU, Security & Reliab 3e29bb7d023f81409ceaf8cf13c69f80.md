# 15.H — Performance, Memory, GPU, Security & Reliability Torture Suite

<aside>
🧪

**Purpose:** destructive verification plan for a professional vector+raster creative application. It assumes Petunia Design Studio can be correct in ordinary use yet still fail through memory/VRAM growth, malformed assets, save corruption, long-session decay, font/profile parsing, stale background results or UI/render synchronization.

</aside>

# 1 — Performance is a correctness property

Petunia is not “fast” because it is written in Rust or uses Slint. Every relevant claim needs workload, build mode, machine, OS, GPU/driver, revision, measurement method and raw/result artifact.

# 2 — Hardware tiers

Define after real measurements:

- **Low-spec target** — modest CPU, 8–16 GB RAM class, integrated/older discrete GPU according to actual supported renderer;
- **Recommended** — mainstream modern desktop/laptop;
- **Heavy-project** — higher RAM/VRAM for stress and production workloads.

Do not publish invented minimums. A configuration becomes officially supported only after evidence.

# 3 — Startup

Measure cold start, warm start and first-run. Separate process launch → window visible → first document/home frame → input responsive. Font scan, plugin discovery, resource validation and thumbnail work should not unnecessarily block time-to-interactive.

# 4 — Idle

With Home and a simple document open, measure CPU wakeups/utilization, RAM, GPU/VRAM and background jobs. Hover and animated chrome must not create continuous busy loops.

# 5 — UI interaction traces

Measure at least:

- menu opening/submenu;
- command palette typing/results;
- Layers scroll/select/rename/drag;
- Properties field edit/scrub;
- panel resize/docking drag;
- Persona switch;
- workspace preset restore;
- font dropdown search;
- asset search/grid scroll;
- continuous window resize.

Record latency distribution/spikes rather than average FPS only.

# 6 — Canvas traces

Pan, zoom, fit, selection marquee, Move transform, Node drag, Pen placement, shape creation, gradient stop drag, snapping-heavy transform, dense-node edit, text edit/caret selection and Surface manipulation.

# 7 — Raster/Photo traces

Brush stroke at representative sizes, pressure input, Eraser, selection preview, adjustment change, mask editing, histogram update, large image zoom/pan and layer compositing.

# 8 — Vector scaling corpus

Create fixtures with increasing object/path/node counts, nesting, masks, booleans, effects, gradients, text and Surfaces. Measure selection/hit-test, Layers updates, scene extraction, render, save and export. Look for complexity cliffs, not only linear averages.

# 9 — Raster scaling corpus

Representative 2K/4K/8K images where supported, multiple pixel layers, 8/16-bit paths, masks and adjustments. Measure tile residency, brush dirty regions, compositing, undo memory and export.

# 10 — Text/font scaling

Large text frames, multilingual scripts, many font families/styles, missing fonts, fallback chains and OpenType features. Measure font enumeration/search, shaping/layout and text editing latency. Font preview list is virtualized.

# 11 — Layers scaling

10k row normal benchmark and 100k synthetic stress where the presentation model can meaningfully represent it. Deep hierarchy and rapid deltas are separate tests. Measure selection, expand/collapse, rename, drag and thumbnail churn.

# 12 — Assets scaling

Thousands of assets/thumbnails. Test cold thumbnail generation, cached reopen, search/filter, category switching, delete/reimport and unavailable files. Background queue is bounded and cancellable.

# 13 — RAM categories

Attribute when possible: canonical document, history, derived evaluation/cache, raster tiles, renderer CPU mirrors, Slint/presentation models, thumbnails/assets, text/font caches, plugins/jobs and temporary import/export buffers.

# 14 — VRAM categories

Render targets, vector scene buffers, raster textures/tiles, thumbnails, glyph atlases, intermediate compositing, masks and staging buffers. Closing a document must release its resources after expected deferred GPU lifetime.

# 15 — Undo memory

History is explicitly benchmarked under geometry edits, transform drags, text edits and raster strokes. One logical stroke/drag is one transaction. Texture changes should avoid whole-image snapshots if the chosen architecture supports dirty-tile/delta history.

# 16 — Allocation audit

Profile hot paths for Vec/String/temp allocations, clones of Document/Mesh/Raster blocks, repeated Slint model conversion, formatting and icon/resource lookup. Optimize measured hot allocation, not stylistic clone avoidance.

# 17 — Long-session soak

Representative loop for practical hours or scheduled duration:

Open PTND → vector edits → text → layer churn → Photo edit → undo/redo → import/place → save → export → Persona switch → workspace change → close/reopen → repeat.

Observe RAM, VRAM, handles, worker/task count, queue sizes, caches, frame/input latency, save latency and log growth.

# 18 — Repetition leak tests

Repeat N times: open/close document; open/close Export; create/delete raster layer; import/delete image; add/remove adjustment; switch workspaces; undo/redo; load/unload plugin/resource pack; create/destroy floating panel; change theme/DPI.

# 19 — Backpressure

Thumbnail, import, export, histogram, font scan, plugin and other jobs have bounded queues. User can cancel where safe. Newer document revision invalidates stale result. Flooding file watcher or asset import does not grow unbounded memory.

# 20 — Renderer synchronization

Every committed ChangeSet produces correct render invalidation. UI-only state must not force full scene rebuild. Panel hover must not trigger geometry evaluation. Renderer cache is disposable and never canonical.

# 21 — Device/surface failure

Inject or simulate adapter/surface/device loss when backend/testing permits. Preserve canonical document, surface actionable error, rebuild resources if supported, or fail safely. Device failure never marks document saved/clean or corrupt.

# 22 — Security assets

Threat assets include user documents, linked assets, system files, credentials if plugins/providers ever introduce them, clipboard contents, plugin permissions, resource packs, user privacy, saved output and application integrity.

# 23 — Trust boundaries

PTND package; legacy migration; SVG/PDF; raster image decoders; TIFF; fonts; ICC profiles; CSV/JSON Data Merge; clipboard; OS drag/drop; linked files; plugin package; Lua scripts; WASI components; themes/icon packs; Help/URL launcher; updater if added.

# 24 — PTND hostile corpus

Truncated ZIP, duplicate entries, absolute paths, ../ traversal, symlink/reparse escape where extraction applies, huge entry count, huge decompressed size, compression bomb, malformed manifest, duplicate IDs, missing resources, unknown schema, future schema, invalid JSON depth/size, invalid extension namespace and inconsistent checksums/lengths where used.

# 25 — SVG hostile corpus

External references, scripts/active content, huge path counts, pathological nested transforms, enormous dimensions, malformed numbers, NaN/Infinity, invalid XML/encoding, referenced files/URLs and resource exhaustion.

# 26 — PDF hostile corpus

Malformed/truncated files, huge object graphs, embedded fonts/images, cyclic references where parser behavior applies, large page counts, unexpected external/action data. Use reviewed parser boundaries and never execute embedded actions/scripts.

# 27 — Image hostile corpus

Huge declared dimensions, decompression bombs, invalid metadata, truncated data, exotic channel/bit-depth combinations, integer overflow sizes and decoder errors. Validate metadata/limits before large allocations where APIs allow.

# 28 — Font hostile corpus

Font files are untrusted. Fuzz/review parser stack, bound metadata/table sizes, isolate source errors and never allow malformed font to crash the whole document/session. Font scanning occurs off UI thread.

# 29 — ICC/profile hostile corpus

Invalid profile size, tags, transforms and unsupported classes. Profiles do not execute code. Invalid profile yields controlled diagnostic and defined fallback/abort behavior without silent color reinterpretation.

# 30 — Data Merge hostile corpus

Very large CSV/JSON, malformed encoding, unexpected columns/types, formula-like text, path/URL content and output filename injection. Binding expressions, if supported, are sandboxed/side-effect-free according to engine contract.

# 31 — Path/file safety

Native paths are not assumed UTF-8. Containment operations define symlink behavior. External resource path resolution cannot escape allowed package/staging roots. Save/export uses safe temp + atomic replace where platform supports it.

# 32 — Disk-full/read-only

Inject save/export failure. Prior valid document remains recoverable, current document stays dirty, error explains target and recovery. Temp residue is cleaned/recoverable.

# 33 — Crash during save

Test failure before temp complete, after complete but before replace, during replace as platform permits, and on next startup. Define which version is authoritative and how recovery detects residue.

# 34 — Clipboard privacy/security

Read clipboard only on direct paste or explicitly authorized operation. Do not poll continuously. Private Petunia payload is versioned/validated. Prefer richest safe supported flavor, not blindly private data. Logs never dump clipboard payload.

# 35 — Drag/drop security

File/URL/text/custom drops are hostile. URL does not auto-fetch network content. Internal drags use opaque session IDs, never pointers. Drop validates before canonical mutation.

# 36 — Plugin security

Plugins receive capability grants, not raw document pointers, Slint objects, GPU resources or unrestricted filesystem/process/network. Capability requests are inspectable and revocable according to accepted plugin contract.

# 37 — Lua

Lua host exposes semantic API only. File/network/process access absent by default unless explicit capability. Limit runaway execution/resources where practical; plugin failure is contained and diagnosed.

# 38 — WASI

Strong-isolation tier uses component/WASI capability model; preopened resources and network permissions are explicit. Component version compatibility and traps are handled without document corruption.

# 39 — Resource packs

No code execution. Validate manifest/schema/version/namespace, alias cycles, token types, SVG active content, file count/size, path containment, locale completeness, contrast-critical tokens and license/provenance. Invalid pack cannot prevent safe-mode startup.

# 40 — Supply chain

Lock dependencies, review native/build-script dependencies, maintain license policy, run vulnerability tooling appropriate to Rust and bundled native artifacts, produce artifact hashes and SBOM/signing when release process adopts them.

# 41 — Unsafe

Pure foundation/domain/application crates should forbid unsafe where feasible. Unsafe is isolated to minimal adapter/backend with written invariants, dedicated tests/fuzzing and owner/rationale.

# 42 — Panic audit

User-controlled document/asset/plugin/UI input paths must not rely on unwrap/expect/index assumptions. Classify every panic/unwrap in critical boundaries as proven invariant, debug-only or remediation.

# 43 — Numerical robustness

Coordinates, transforms, paths, color values, percentages and raster sizes validate finite/range semantics. NaN/Infinity/zero denominators/overflow/near-singular transforms are rejected or handled through explicit domain behavior.

# 44 — State integrity

Selection never points to removed object. Layer tree and canvas share canonical identity. Renderer/presentation caches never become source of truth. Undo branching invalidates redo correctly. Background result cannot commit to stale revision.

# 45 — Recovery

Every long/high-impact operation declares cancelability, retryability, idempotency, partial-state behavior, cleanup and resume/rollback. “Try again” must not duplicate external output.

# 46 — Observability

Stable diagnostics and structured performance events without private reasoning. Log levels, privacy/redaction, rotation/size limits and developer tracing are explicit. Per-frame spam is prohibited by default.

# 47 — Support bundle

May include version/build, OS, GPU/driver/backend, sanitized preferences, relevant diagnostics, profiler summary and project metadata—not full project content, clipboard, linked files or user secrets unless explicit consent/export.

# 48 — Release gate

No known data-loss path; no uncontrolled memory/VRAM growth; no critical malformed-input crash; no unsafe path traversal; no shell stall violating accepted hard budget; no stale background mutation; PTND atomic-save/recovery evidence current; final packaged artifact tested from clean state.