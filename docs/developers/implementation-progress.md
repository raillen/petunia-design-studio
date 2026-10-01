# Implementation execution ledger

**Scope: Milestone Required (MVP) and V1 Required.** The user authorized execution of the [roadmap](/developers/implementation-roadmap-2026-09-30). Work started from the audited baseline `73447647a34f5b4be209415c36e6965c5f71b9b9`; baseline findings and probes remain historical evidence. This ledger describes implementation, not a release announcement. **MVP, V1 and the complete roadmap remain unfinished.** Future candidates/research retain their admission criteria.

## Current implementation

| Delivery | Status | Implemented behavior and remaining work |
| --- | --- | --- |
| M0.1 reproduced correctness bugs | Implemented; automated gate passed | Fresh revision-zero R-tree; flattened caches own their generation; boolean empty identities; finite-chord RDP; compound offsets retain components; actual eraser; partial-alpha blending; UTF-8 color safety; object alpha applied once. Corpus and visual coverage remain broader than these regressions. |
| M0.2 schema/integrity | In progress | Schema 3/local paths and modifier reference frames, explicit parent-input migration, atomic path/frame edits, iterative graph validation, finite metrics/references, reversible detachment of text/bindings, validated live chains and restricted mutable handles. Raw serde decoding still requires validation at trust boundaries; resource limits and a broader hostile-input corpus remain. |
| M0.3 transactions/history/save | In progress | Conflicts fail; staging/replay failures publish nothing; failed undo/redo retains entries; bounded history and content save points; exclusive atomic synced package saves and bounded ZIP reads. Operation revisions/preconditions, complete COW snapshots, resource packaging, recovery/crash/disk-failure proof remain. |
| M1 scene/render/worker architecture | Pending; isolated fixes implemented | Metadata cache hits avoid cloning paths; even-odd cached hit preserves holes; raster storage/upload alpha agrees; 16-bit bytes are little-endian; brush work/addresses are checked; cancelled jobs remain terminal. Shared render scene, proper composition, image decode cache, tile workers and actual job execution remain. |
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

## Verification and practical limits

`cargo xtask verify` passed: 661 tests, no failures or ignored tests; 57 focused regressions/checks were added, plus a new shell pointer regression and two compile-fail API boundary doctests. The modifier-frame continuation added 15 regression cases in total. Strict formatting, Clippy and forbidden dependency-edge checks passed. The machine-readable checks record exact commands, results and totals; tests are headless, including Freya TestingRunner interaction fixtures and independent Poppler PDF parsing. Strict docs keep EN/pt-BR parity and dead-link checking. A debug compositor test now checks pixels/culling rather than a fragile 50 ms wall-clock limit; performance belongs to controlled release benchmarks.

The benchmark now honors `--objects`, `--iterations`, `--warmup` and `--json`. JSON separates fixture build, cold snapshot and warmed query mean/p50/p95/p99, with platform/profile and scene digests. It excludes painting/GPU/presentation and memory profiling; small smoke runs cannot establish tail latency or FPS. A defensive staging copy introduced during this work made the 10,000-object fixture take about 40 seconds; audited in-place primitives reduced it to about 249 ms with the same scene digest. These uncontrolled runs diagnose that internal regression, not a product performance guarantee.

The cloud linker exhausted the 32 GiB disk after repeated builds. Only generated incremental/app build artifacts were removed; the source, audit evidence and artwork were preserved. Failed checks before corrections are not final acceptance evidence.

There is no graphical desktop/tablet/physical print validation in this run. Immutable path buffers are shared; document-container/image copies and encoded history sizing remain provisional for large projects. Prumo CLI is absent, so this ledger tracks scope/evidence without changing locked Prumo goal files or claiming CLI execution. ICC/profile licensing, fonts/RIP, Wayland/X11, accessibility and user-task gates require their declared fixtures/equipment and remain open.

Next dependency: finish M0 resource/mutation/limit contracts, then implement the shared scene and faithful render/export composition before enabling the remaining tools or claiming MVP completion.
