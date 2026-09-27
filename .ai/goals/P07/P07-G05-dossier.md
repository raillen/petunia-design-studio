# P07-G05 Dossier — Canvas World Frame and Reproducible Performance

## Status / scope

- **Goal:** `P07-G05`.
- **State:** `EXECUTING`; the coordinate ADR is `ACCEPTED_V1`; the domain resolver, world-aware cache/index paths, selection preview and Freya snapshot slice are implemented, while release benchmarks and the full vector/text renderer remain open.
- **ADR:** `docs/09 32 — Coordinate Spaces, World Geometry and Migration ADR.md`; pt-BR mirror: `petunia-design-studio/09 32 — Coordinate Spaces, World Geometry and Migration ADR pt-BR.md`.
- **Scope:** `V1_REQUIRED` foundation for the Freya canvas. This is a semantic coordinate/evaluation contract, not a visual-parity claim.
- **Worktree:** `/home/raillen/Documentos/petunia-freya-tools-integration`.
- **Shell authority:** Freya is the active worktree adapter. Slint remains historical/reference material in this worktree.
- **Current blocker:** Prumo `docs audit` cannot open `docs/contracts/builtin.json`; the missing contract is recorded and must be resolved before documentation verification is claimed complete.

## Goal and user outcome

Make selection, transform, culling, handles, preview, scene export and the Freya canvas use one explicit coordinate model. A user can move or rotate a nested/rotated object and see the same world-space geometry in the canvas, hit test, selection frame, preview and export. A developer can reproduce the result from synthetic fixtures and release measurements.

## Non-goals

- No new GPU backend, Vello/wgpu selection, WebView, Slint implementation or generic editor framework.
- No claim of Affinity/Krita/Figma/Inkscape visual parity in this gate.
- No text shaping, raster tiles, effects pipeline, full mask compositor, typed text or image rendering.
- No silent reinterpretation of existing v1 documents or in-place migration of package files.
- No global `cargo fmt` or destructive Git operation; the worktree has unrelated local changes.

## Canonical references

- `docs/09.2` — Canonical Document Model, Object Graph, Resources and Validation.
- `docs/09.4` — Evaluation Graph, Derived Data, Invalidation and Cache Architecture.
- `docs/09.5` — Vector Geometry Engine Numeric Policy, Paths, Boolean, Stroke and Snapping.
- `docs/09.10` / `docs/09.10.1` — Native File Format, Serialization and Open Interoperability.
- `docs/09.21` — Architecture Governance, ADRs, Compatibility and Definition of Done.
- `docs/09.26` — Open ADR Registry.
- `docs/09.27` — UI-Agnostic Core, Ports and Adapters.
- `docs/10.1` — Selection, Transform, Arrange, Align, Distribution and Snapping.
- `docs/10.5` — Layers, Groups, Clips, Masks, Symbols and Styles.
- `docs/10.7` — Surfaces, Artboards, Pages, Guides, Margins and Lightweight Layout.
- `docs/08.23` — Tool Interaction Grammar, Context Toolbar, Modifiers and Canvas HUD.
- `docs/14.1` — Test Architecture and Regression Policy.
- `docs/14.2` — Performance Contract and Benchmark Corpus.
- `docs/14.3` — Golden Project, Fixture and Reproduction Corpus.
- `docs/14.6` — Native Format Migration and Recovery.
- `docs/14.7` — UI/UX Evidence and Semantic Goldens.
- `docs/14.8` — AgentOps, cargo xtask, CI Commands and Evidence Bundles.
- `docs/12.4` and `docs/12.10` — implementation dossier and Prumo workflow.
- ADR to add: `docs/09 32 — Coordinate Spaces, World Geometry and Migration ADR.md`.

## Architecture / dependency map

```text
Surface/Object canonical model
        ↓
Document world geometry resolver (application/domain)
        ↓
GeoCache + SpatialIndex + selection/hit/snap (application)
        ↓
CanvasSceneSnapshot (toolkit-neutral shell/application DTO)
        ↓
Freya adapter (presentation only)
        ↓
Element tree / headless structural evidence

The reverse direction is forbidden: domain/application never imports Freya,
Slint, GPU, Vello, wgpu or window types.
```

### Coordinate contract

- **Local:** path/point coordinates in the object's own geometry frame, before placement.
- **Parent placement:** `bounds.x/y` is the object frame origin relative to its parent; rotation is part of the local placement transform.
- **World/pasteboard:** `W_parent * (T(bounds_origin) * R(rotation))` applied to local geometry.
- **Screen:** `ViewportCamera` projection, consumed only by the canvas adapter.

Invariants:

- `base_path_local()` does not apply `bounds` or `rotation`.
- `evaluated_path_local()` applies modifiers in local space.
- `base_path_world(id) = W(id) * base_path_local()`.
- `evaluated_path_world(id) = W(id) * evaluated_path_local()`.
- placement bounds, local evaluated bounds and world AABB are distinct derived values.
- no consumer applies `bounds`/rotation twice to a world path.
- v1 paths/modifiers without an explicit frame are not guessed; migration reports ambiguity.

## Affected crates / modules

- `petunia_design_foundation`: schema version/diagnostic IDs if the migration version is introduced.
- `petunia_design_geometry`: local/world transform and AABB helpers, tolerance-qualified operations.
- `petunia_design_document`: canonical object frame, world resolver, grouping/reparent and modifier frame policy.
- `petunia_design_application`: revision-aware local/world caches, spatial index, selection/hit/marquee and snapshot query.
- `petunia_design_shell`: toolkit-neutral scene snapshot and coordinate-space overlay DTOs.
- `petunia_design_io`: SVG/PDF and import/export consumers after local/world APIs are established.
- `apps/petunia-design`: Freya presentation adapter; it consumes the snapshot and does not own document geometry.
- `petunia_design_testkit` (new, `publish = false`): fixtures, replay, metrics and release benchmark runner.
- `xtask`: real `bench-smoke` and canvas evidence commands.

## Data model / schema / migrations

- Current native schema is version 1.
- Any new persisted frame interpretation requires a later schema version and an explicit forward migration.
- Migration runs on a staging copy, reports diagnostics, preserves the source package and never edits in place.
- Legacy ambiguous paths/hierarchies fail preflight or receive a bounded repair policy; no heuristic coordinate subtraction.
- Existing APIs remain `DEPRECATED` only for the migration window; new consumers use canonical local/world APIs.
- Scale/shear reparent is rejected until the persisted model can represent it safely.

## Actions / Commands / Transactions

- Existing `Action → Command → DocumentMutator → ChangeSet` remains the only mutation lane.
- Preview state is transient and never enters history.
- Select gesture captures selected IDs, transforms, frame/pivot, coordinate space, modifiers and document revision.
- Commit emits one coherent `ChangeSet` and one undo entry; stale revision fails or safely rebases only for a specified operation.
- Tools report their coordinate space in semantic metadata; UI/MCP do not infer it from pixel values.

## Jobs / concurrency / cancellation

- Cold spatial/world caches are derived and rebuildable.
- Any asynchronous world/scene evaluation captures a revision/generation and discards stale completion.
- Bench/test fixtures are deterministic; wall-clock only appears in release benchmark metadata, never as document truth.
- A future double-click/replay clock is virtual in testkit, not `Instant::now` or `sleep`.

## Security / hostile inputs

- Reject non-finite path/transform coordinates at command/import boundaries.
- Reject dangling parents, duplicate IDs, cycles, incompatible reparent and non-invertible transforms with structured diagnostics.
- Migration preserves unknown/forward fields according to the compatibility policy; it never silently discards them.
- Fixtures are synthetic/minimized and contain no credentials, private keys or user artwork.

## UI / UX / accessibility / localization

- Freya is the active presentation adapter; the scene snapshot is toolkit-neutral.
- Screen/document/parent/local spaces are explicit in overlays and selection handles.
- Pointer, keyboard, focus, a11y and semantic Action/Command behavior remain shared across UI, MCP and plugins.
- No new user-facing strings or tokens are introduced by the coordinate contract; any later HUD copy requires en-US and pt-BR catalog updates.

## Plugin / MCP impact

- No plugin permission or protocol change in the first geometry slice.
- Plugin/MCP queries must use the shared semantic selection/snapshot/preview contracts and report coordinate space.
- A future public frame API is additive and versioned; persisted schema changes require migration notes.

## Performance and memory budgets

No final numeric budget is frozen before the first release baseline. Record:

- world evaluation cold/warm;
- spatial index cold/warm;
- hit, marquee and lasso;
- move/resize/rotate preview;
- commit, undo and snapshot;
- Freya harness separately from the CPU reference compositor;
- p50/p95/p99, warmup, sample count, fixture digest, seed, viewport, zoom, profile, toolchain, OS/CPU, cache size and memory where measurable.

The current report values are historical/debug indicators, not product claims.

## Test / fixture / replay / gauntlet plan

- Unit: local path, world transform, AABB, finite values, modifier local-space operations.
- Contract: `W_before ≈ W_after` for group/ungroup/reparent, no double transform, migration diagnostics.
- Property: affine composition, bounds containment, revision/cache invalidation, random valid TRS.
- Headless: nested/rotated selection, hit, culling, handles, marquee, lasso, preview/commit/cancel, stale revision.
- Structural golden: world path, painter order, selected/hovered state and overlay coordinate space.
- Replay: virtual-clock pointer/tool trace, stable document digest/revision/selection.
- Visual golden: only after the Freya testing API is verified to capture a framebuffer; never substitute the CPU compositor silently.
- Benchmark: release runner with nearest-rank percentiles and machine metadata; `xtask bench-smoke` must execute a real scenario.

## Documentation impact manifest

- `docs/09 32 — Coordinate Spaces, World Geometry and Migration ADR.md` (new, en-US).
- `petunia-design-studio/09 32 — Coordinate Spaces, World Geometry and Migration ADR pt-BR.md` (new mirror).
- `docs/09.4`, `09.5`, `10.1`, `10.5`, `10.7`, `08.23` (cross-links/clarification).
- `docs/09.21`, `09.26` (ADR governance and registry consistency).
- `docs/09.10`/`14.6` (migration/version policy).
- `docs/14.1`/`14.2`/`14.3`/`14.7`/`14.8` (evidence and benchmark policy).
- `docs/AUTHORITY_MAP.json` and `docs/contracts/bindings.json` only after the generated/documented contract is reconciled.
- `docs/contracts/builtin.json` is currently missing; Prumo documentation verification is blocked until its source is restored/accepted.

## Implementation log / evidence

- 2026-09-25: Plan approved after repository audit; no production code changed by this dossier.
- 2026-09-25: Prumo v0.6.0 goal `P07-G05` created and moved `DRAFT → PLANNED`.
- 2026-09-25: `prumo docs audit` failed because `docs/contracts/builtin.json` is absent; documentation verification remains blocked until that source is restored or explicitly accepted.
- 2026-09-25: Implemented explicit `base_path_local`, `evaluated_path_local`, world-transform and world-bounds APIs; legacy readers remain available during v1 migration.
- 2026-09-25: Added revision-aware legacy/local/world cache entries, world-space polygon/hit/sample queries and world-AABB spatial culling paths; ambiguous legacy paths/modifiers remain explicit diagnostics.
- 2026-09-25: Migrated grouping/reparent composition to preserve `W_before ≈ W_after` for representable TRS frames; clip-group placeholders retain a separate no-geometry path.
- 2026-09-25: Migrated Select hit-test, marquee/lasso bounds, handles and transform preview to the world-aware resolver; stale previews and stale commits are rejected.
- 2026-09-25: Extracted `petunia_design_shell::canvas::CanvasSnapshot` and made the Freya adapter consume it instead of owning a private scene projection.
- 2026-09-25: Added `canvas_snapshot_uses_world_frame_and_rotation`, proving a rotated 40×30 frame at `(10,20)` projects to world AABB `[-20,20,30,40]`.
- 2026-09-25: Validation passed: shell/app `cargo check`; 102 `tools_test` tests; 43 document tests (39 unit + 4 property); the focused snapshot regression passed; `git diff --check` passed.
- 2026-09-25: Added `petunia_design_testkit` (`publish = false`) with a deterministic scene fixture builder, three fixture contract tests (determinism, nested world anchors, and a real Select/Transform preview gesture), a headless release benchmark runner and a real `xtask bench-smoke` execution.
- 2026-09-25: The first release benchmark run was invalidated: a test binary had 155 MB of zero-filled header bytes (`Exec format error`, os error 8) caused by concurrent Cargo file-lock contention. Resolved by `cargo clean -p petunia_design_testkit` and sequential execution; no source change was involved.
- 2026-09-25: Found and fixed a methodology defect in the benchmark: several scenarios executed a whole anchor loop per sample, so raw p50 values were not comparable. Every scenario now reports `operations_per_sample` and `p50_us_per_operation`.
- 2026-09-25: Release baseline recorded in `PERF_REPORT_2026-09-23.md` §8 for 500/2.000/10.000 synthetic objects. `canvas-snapshot` is the dominant cost (108 µs → 445 µs → 3.159 ms) and `select-marquee` is the worst single gesture (18,3 ms in 10k, above a 60 fps budget). `transform-preview-event` is flat at 0,07 µs, proving the new preview DTO and revision guard are not the bottleneck.
- 2026-09-25: Decision recorded: do not select a GPU backend yet. A scene snapshot cache plus dirty-rect/culling outranks Vello/wgpu, because the measured waste is a linear CPU snapshot rebuilt every frame.
- These results prove domain/application contracts, shell integration and structural scene data only. They do not prove visual parity, release performance, framebuffer output, text shaping, image/effects rendering or full Photo interaction.

## Known risks / open work

- The Freya renderer still paints parametric object proxies from frame bounds, fill and shape. It does not yet render the explicit `GPath`, text runs, strokes, gradients, images, effects or complex masks in the canvas.
- The current DTO is sufficient for a headless semantic oracle, but it is not yet a painter-order-complete scene contract for every document object kind.
- The current world cache is revision-keyed and memoized, but no release benchmark, p50/p95/p99 profile, memory baseline or input-to-frame harness exists.
- Existing v1 paths and modifiers without a persisted coordinate frame remain ambiguous by design; they require an explicit migration policy rather than heuristic interpretation.
- The Prumo `docs/contracts/builtin.json` source is missing, so documentation audit cannot be called green.
- Historical Slint/GPUI references remain in the notebook; Freya worktree updates must record supersession rather than globally rewriting historical documents.

## Handoff / next safe step

Keep `P07-G05` in `EXECUTING`. The fixture and release benchmark package is
done; the next bounded package is reordered by the measured baseline in
`PERF_REPORT_2026-09-23.md` §8:

1. cache the scene snapshot per document revision plus viewport, and reuse it
   across frames, so `canvas-snapshot` stops being O(n) per frame;
2. split the snapshot cost by object kind so painting, culling and DTO
   construction are measured separately;
3. add a real headless Freya framebuffer/structural golden, only after the
   Freya testing API is verified to capture output;
4. reduce the `select-marquee` candidate set with the spatial index and a
   quantised view query, instead of testing every object per event;
5. only after those, choose the vector renderer strategy and migrate explicit
   paths/text/appearance incrementally;
6. keep the Prumo documentation blocker and bilingual synchronization visible
   until resolved.

Do not start Vello/wgpu, text shaping or image/effects work in this goal: the
measured bottleneck is a linear CPU projection rebuilt per frame, which a
faster renderer would only hide.
