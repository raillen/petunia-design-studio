# Implementation execution ledger

**Studio update (2026-10-04):** [ADR-012](/developers/adr/ADR-012-studio-workspace) and the [UI/UX audit](/developers/uiux-studio) define the desktop redesign. This wave has its own final results in [uiux-studio.json](/implementation/uiux-studio.json); the ADR-011 validation below belongs to the merged baseline.

## 2026-10-03: native CMYK continuation

[ADR-011](/developers/adr/ADR-011-native-cmyk-raster) introduces five-lane 8/16-bit raster, immutable ICC assignment/conversion, schema 6/index 3, native painting/fill, editable TIFF placement, layer TAC/separation inspection, native layer TIFF and ICC CMYK PDF. UI/CLI integration is implemented. Regular PDF requires a common press profile across included CMYK surfaces/layers/images; differing profiles require explicit conversion. Whole-page ink proof/overprint, DeviceLink, PDF/X-4 and full release acceptance remain open.

**Final automated evidence:** after implementation, `cargo xtask gauntlet` passed **930 workspace tests**, with zero failures/ignored tests, formatting, strict Clippy, architecture, CLI conformance and four project scenarios. `cargo xtask ui-gauntlet` passed **64 desktop tests**, already included in the total. Documentation passed with **33/33 EN/pt-BR pages** and zero dead links. Independent pypdf/Poppler verification confirmed exact ink, alpha and original ICC bytes in four layer/image PDFs at 8/16-bit. [Execution ledger](/implementation/native-cmyk-v1.json) records logs, captures, artifact hashes and limits. The 904-test result below belongs to the merged ADR-010 baseline.

## Historical: merged ADR-010 correction contract

This update supersedes the older unvalidated continuation summaries below. ADR-010 implements bounded text, history, histogram, ICC and PDF corrections. Final validation begins only after the implementation phase. Actual commands/results are recorded in `/implementation/mvp-v1-corrections.json`; no result is inferred from source presence. MVP human/hardware acceptance and the complete V1 scope remain open.

**Final automated evidence:** `cargo xtask gauntlet` passed: **904 workspace tests**, zero failures/ignored tests, formatting, strict Clippy, architecture, CLI conformance, four project fixtures, 32/32 EN/pt-BR pages and the VitePress dead-link build. `cargo xtask ui-gauntlet` passed 62 desktop tests (included in the workspace total). [Execution ledger](/implementation/mvp-v1-corrections.json) contains logs, artifact hashes and acceptance limits.

[ADR-010](/developers/adr/ADR-010-text-histogram-icc-and-pdf)

**Scope: Milestone Required (MVP) and V1 Required.** The user authorized execution of the [roadmap](/developers/implementation-roadmap-2026-09-30). Work started from the audited baseline `73447647a34f5b4be209415c36e6965c5f71b9b9`; baseline findings and probes remain historical evidence. This ledger describes implementation, not a release announcement. **MVP, V1 and the complete roadmap remain unfinished.** Future candidates/research retain their admission criteria.


## Historical MVP record: persistent pixels and native workflows (before final validation)

[ADR-009](/developers/adr/ADR-009-persistent-raster-and-native-workflows) introduces schema 4 binary PTND resources and editable sparse RGBA/coverage planes, including opaque masks without full-plane allocation. Brush/eraser/selection/fill publish single guarded transactions; original images remain unchanged. Clip-group alpha is applied once. Tile payloads and sources are shared across snapshots/history/duplication and admitted by explicit budgets.

Desktop I/O/codec work now runs in a bounded worker with stable tab/revision guards. Saves acknowledge the captured history state, preserving newer dirty edits; Close All waits for every required save. Atomic native/export writes coordinate cooperating processes through stable sidecar locks. Recovery stores binary snapshots and offers restoration at startup into a new dirty tab. Native Linux object clipboard uses wl-clipboard/xclip; subtree copy/duplicate/delete remaps topology and origin-aware paste, and cut waits for ownership before deleting.

Uniform text styles and frame/artistic flow persist. Worker-prepared glyph metadata supplies artistic overflow selection and missing-font/overset diagnostics. Native multiline drafts supply text editing/IME; exact canvas caret integration is still unfinished. SVG output consumes the shared scene and preserves supported masks/paints/transforms/glyph ink and embedded 8/16-bit pixels; strict basic SVG input rejects unavailable constructs explicitly. ICC RGB image derivatives use bounded moxcms conversion to sRGB without rewriting originals. PNG export has a real transparent artwork preview and cancellation. PDF remains outside the MVP chooser with its V1 reason.

**Status recorded before final validation on 2026-10-03.** Additional meaningful regression sources were written, without running tests, builds or gates. The generated reference/check ledger distinguishes source counts from results. The original exit gate remains open: in-canvas text caret/IME, native tablet pressure, display-profile setup, real histogram composition, aggregate cache/history performance, four task projects and Linux/backend/accessibility/install evidence still need completion. The cloud cannot supply physical tablet/calibrated-display/user evidence. Implementation of these subsystems does not close the complete MVP or V1.

Earlier continuation sections below are historical wave records; ADR-009 supersedes their schema/resource/ICC-input and workflow pending statements.


## MVP continuation: shaped text and worker canvas (validation deferred)

[ADR-006](/developers/adr/ADR-006-shaped-text-and-canvas-preview) connects advanced uniform-style text shaping and TTF/CFF ink outlines to the shared CPU scene. Glyph clusters/fallback families, width wrapping and tracking are retained; fills/strokes/masks use nonzero winding for text. Originals remain editable. Empty text stays empty; live crop/effects/group transforms compose through the existing backend. Prepared text has bounded LRU residency and pinned-owner accounting.

The canvas now presents worker-composed frames from that scene; cold image/text scene preparation leaves painting. Immutable sources are reused across selection/camera changes, with unique process-local identities across sessions. The latest-request controller cancels replacements, checks source/revision/camera/channel/proof before publication and projects a retained same-source camera frame at its original world region. Skia retains one completed-frame upload; the obsolete flat artwork painter/default-font text renderer and per-image upload cache were removed. Job metadata lookup/history and canceled queue admission are bounded. Common-backdrop composition is shared with PNG; backdrop differences remain explicit. Soft proof requires an available ICC CMM.

**34 regression cases were prepared, none executed.** This continuation, the preceding 37 render/worker cases and 35 image cases remain unvalidated. Tests/builds/gates/CI are still deferred by the user's instruction. M1/M2 remain open for styled text/caret/graphemes/IME/text-on-path, coherent hit-testing/overflow, total budgets, tiled presentation and measured acceptance. M0 resources/COW/recovery, persistent bitmap layers and M4 Linux/product gates remain unfinished; V1 ICC/CMYK/PDF are unchanged.

## MVP continuation: image sources and composition (validation deferred)

[ADR-005](/developers/adr/ADR-005-immutable-image-assets) adds immutable content-keyed originals; bounded PNG/JPEG/WebP/RGB-gray TIFF decoding; preserved gray/gray-alpha RGBA16 little-endian samples and EXIF orientation; shared LRU/negative caching with pinned-owner residency accounting; linear-light premultiplied area mipmaps; and CPU image sampling through transforms, live crops, masks, spatial transparency, adjustments and available effects. Placement now requires a valid admitted source before any document/selection/history change. The localized import dialog displays failures. Skia reuses bounded uploads and honors rotation/opacity; the sampled display histogram reuses prepared pixels, including 16-bit sources. Originals remain embedded and native schema 3/wire arrays remain unchanged.

The current image continuation is **unvalidated**. Regression sources are prepared without executing tests or gates. ICC-tagged sources, CMYK/HDR, animated PNG/WebP and declared non-sRGB PNG color conversion return capability reasons; a full ICC CMM remains V1 work. Image perspective/contour resampling, native file picker/portal, worker-driven cold import admission, remaining scene capabilities and resource packaging remain pending. ADR-006 supplies worker-driven canvas presentation. Display/export derivatives are RGBA8; this does not make persistent full-precision pixel layers available.

## MVP continuation: render snapshots and workers (validation deferred)

[PR #3](https://github.com/raillen/petunia-design-studio/pull/3) is a draft. [ADR-004](/developers/adr/ADR-004-render-snapshots-and-workers) adds immutable local/world render snapshots; actual antialiased vector coverage; multiple spatial fills/strokes; isolated opacity/blending; vector/alpha/luminance masks; spatial transparency; ordered blur/drop-shadow passes; linear-RGB tonal adjustments with prepared curves; old/new scene damage; direct artboard PNG rendering with DPI; and real bounded, cancellable workers with revision-tagged results. Application scheduling can render an owned snapshot in the background. Immutable encoded image sources now share buffers across snapshots/history/duplication without changing the native wire format. Compatibility APIs now return capability errors.

These are new implementations with prepared regression sources, not verified features. At the user's explicit request, builds, tests, lint/format checks, conformance, migrations and documentation validation are deferred until all MVP features have been implemented. Draft continuation commits skip automatic CI during this phase. Historical results below apply to foundation commit `9fb1dc22c25cc4261168ec0df3f5a5b00b644264`, not the current PR head. The next gate must include the new source, lockfile edges and regression cases.

M1 remains in progress: Persistent/local color-paint frames, remaining glyph/style capabilities, worker-driven import admission, total geometry/work budgets, tiled workers and visual/performance acceptance remain open. ADR-006 implements shared CPU canvas presentation and bounded shaped-outline preparation; final acceptance remains open. Unsupported content receives a reason instead of fabricated rectangles. M0 resource packaging/COW/recovery, the complete M2/M3 tools and text/bitmap integration, and M4 Linux/user-task release gates also remain open. V1 ICC/CMYK and professional PDF have not been implemented by this continuation.

## Current implementation

| Delivery | Status | Implemented behavior and remaining work |
| --- | --- | --- |
| M0.1 reproduced correctness bugs | Implemented; automated gate passed | Fresh revision-zero R-tree; flattened caches own their generation; boolean empty identities; finite-chord RDP; compound offsets retain components; actual eraser; partial-alpha blending; UTF-8 color safety; object alpha applied once. Corpus and visual coverage remain broader than these regressions. |
| M0.2 schema/integrity | In progress | Schema 3/local paths and modifier reference frames, explicit parent-input migration, atomic path/frame edits, iterative graph validation, finite metrics/references, reversible detachment of text/bindings, validated live chains and restricted mutable handles. Raw serde decoding still requires validation at trust boundaries; resource limits and a broader hostile-input corpus remain. |
| M0.3 transactions/history/save | In progress | Conflicts fail; staging/replay failures publish nothing; failed undo/redo retains entries; bounded history and content save points; exclusive atomic synced package saves and bounded ZIP reads. Operation revisions/preconditions, complete COW snapshots, resource packaging, recovery/crash/disk-failure proof remain. |
| M1 scene/render/worker architecture | In progress; current continuation unvalidated | Metadata cache hits avoid cloning paths; even-odd cached hit preserves holes; raster storage/upload alpha agrees; 16-bit bytes are little-endian; brush work/addresses are checked; cancelled jobs remain terminal. New snapshot/backend/worker implementation is described above. Shared scene/worker presentation and a shaped-outline cache are implemented by ADR-006, unvalidated; remaining capabilities, import admission, total budgets, persistent tiles and acceptance remain. |
| M2 vector/text | Pending integration gates | Pen/pencil/node/knife/builder read parent projections and atomically publish path edits; perspective/transparency/crop convert world inputs to local frames. This migration is tested by the existing tool workflows; world-space editing under all group/rotation cases, typography/glyph runs/caret/IME and full usability still require work. |
| M3 bitmap/selection/adjustments | Pending | Correct brush kernels do not make persistent PixelLayer/MaskLayer, Photo tools and selection-aware painting available. Source preservation, live adjustments and resource/tile round trips remain. |
| M4 files/Linux/MVP release | Pending | Rust toolchain is pinned; strict lint issues corrected; migrations gate is executable; PDF tests use Poppler; benchmark arguments and snapshot percentiles are implemented. Packaging, actual pressure/IME/clipboard/a11y, recovery, SVG/PNG fidelity and complete user tasks remain. |
| V1-A precision/assets/professional UX | Pending | Full professional tool column, text/styles/assets/symbol instances, preflight and plugins/MCP quotas/interoperability remain. |
| V1-B true ICC/CMYK | Pending | ICC CMM, typed ink/profile data, true proof/separations/TAC, monitor configuration and independent validation remain V1 Required. No current heuristic has been promoted to an ICC proof. |
| V1-C professional PDF | Pending; page/preflight corrections implemented | Enabled surfaces only, actual page dimensions and local page origin; strict mode rejects approximation/unknown color; export inclusion has a reversible command. Glyph embedding, images, shared world scene, clips/groups, ICC OutputIntent, spots/overprint, PDF/X-4 and independent print validation remain. |
| V1-D release and future versions | Pending | Professional projects, long sessions, compatibility corpus, hardware/print and installation evidence remain. Future research/candidates have not been silently declared shipped. |

Contracts: [ADR-003](/developers/adr/ADR-003-local-modifier-frames) and [ADR-002](/developers/adr/ADR-002-local-path-and-integrity). Generated [source/test reference](/implementation/contracts.json) and [check evidence](/implementation/checks.json) are reviewable alongside the patch.

## MVP continuation: modifier frames

Milestone Required: [ADR-003](/developers/adr/ADR-003-local-modifier-frames) adds schema 3 with explicit local reference sizes and migration of schema 1/2 modifier inputs. Crop/perspective/transparency follow placement and nonuniform resize without rewriting source/parameters; contour evaluates in its reference frame. Disabled entries migrate too. Local/world opacity samples pull back into that frame and do not allocate/sort stops per sample.

Bake prepares and validates the candidate before publication, preserves rotated world geometry and surviving masks, and avoids applying following modifiers twice. An empty vector crop remains empty while retaining the source. Selective contour bake after another geometry operation fails with a reason to use full geometry bake. Rotated world crop requires a polygon mask and remains unavailable, rather than substituting an AABB. Existing preview/export still needs the shared scene and full spatial mask rendering.

The full gate exposed a ruler bug: guide IDs based on milliseconds collided during fast consecutive gestures. A domain CreateGuide command now allocates unique IDs and history preserves them. Shape fixtures now explicitly create rectangles instead of relying on shapeless-object fallback geometry. A pointer regression proves rotated perspective preserves untouched corners and undo restores the original document.

## Finding disposition

F02/F03/F09/F18/F19/F20 have targeted corrected results. F01/F10/F11/F12/F14/F17/F21–F25/F28/F29/F35/F37 have implementations addressing specific defects; the full finding/milestone remains open where its broader acceptance is unmet. Other findings remain pending. Releasing the program requires the original roadmap gates, not merely a handler or a green dispatch test.

Additional defect discovered during implementation: tuple data formatters `Prefix`, `Suffix` and `NumberDecimals` could not serialize under internally tagged serde. Named wire fields now round trip all variants while preserving legacy unit/Currency representation. Integrity validation also required deleting a path to clear text attachments and data bindings reversibly.

## Historical foundation verification and practical limits

`cargo xtask verify` passed: 661 tests, no failures or ignored tests; 57 focused regressions/checks were added, plus a new shell pointer regression and two compile-fail API boundary doctests. The modifier-frame continuation added 15 regression cases in total. Strict formatting, Clippy and forbidden dependency-edge checks passed. The machine-readable checks record exact commands, results and totals; tests are headless, including Freya TestingRunner interaction fixtures and independent Poppler PDF parsing. Strict docs keep EN/pt-BR parity and dead-link checking. A debug compositor test now checks pixels/culling rather than a fragile 50 ms wall-clock limit; performance belongs to controlled release benchmarks.

The benchmark now honors `--objects`, `--iterations`, `--warmup` and `--json`. JSON separates fixture build, cold snapshot and warmed query mean/p50/p95/p99, with platform/profile and scene digests. It excludes painting/GPU/presentation and memory profiling; small smoke runs cannot establish tail latency or FPS. A defensive staging copy introduced during this work made the 10,000-object fixture take about 40 seconds; audited in-place primitives reduced it to about 249 ms with the same scene digest. These uncontrolled runs diagnose that internal regression, not a product performance guarantee.

The cloud linker exhausted the 32 GiB disk after repeated builds. Only generated incremental/app build artifacts were removed; the source, audit evidence and artwork were preserved. Failed checks before corrections are not final acceptance evidence.

There is no graphical desktop/tablet/physical print validation in this run. Immutable path buffers are shared; document-container copies and encoded history sizing remain provisional for large projects. Prumo CLI is absent, so this ledger tracks scope/evidence without changing locked Prumo goal files or claiming CLI execution. ICC/profile licensing, fonts/RIP, Wayland/X11, accessibility and user-task gates require their declared fixtures/equipment and remain open.

Next dependencies: finish M0 resource/COW/recovery contracts, complete persistent bitmap layers/masks and remaining text/editing capabilities against the shared scene, then run all deferred gates and original Linux/product acceptance before claiming MVP completion.
