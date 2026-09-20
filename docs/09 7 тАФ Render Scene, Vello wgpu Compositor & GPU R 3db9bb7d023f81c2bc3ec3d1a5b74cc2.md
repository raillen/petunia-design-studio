# 09.7 — Render Scene, Vello/wgpu Compositor & GPU Resource Lifecycle

# Separation

Document → Evaluation → RenderScene → backend(s). RenderScene is reconstructible and contains evaluated primitives/resources, not edit-time business logic.

# Scene primitives

Vector path draw, glyph run, raster image/tile, clip, mask, isolated group, filter/effect surface, compositing operation and overlays in a separate editor-overlay layer.

# Vector path

Vello is primary 2D vector/text display backend. Aubrieta does not serialize Vello Scene/Brush/Path types.

# Compositor

wgpu compositor defines group isolation, opacity, blend mode, clip/mask ordering, intermediate surfaces, raster effects and vector+raster convergence. Blend semantics must be specified in a backend-independent reference.

# Resource lifecycle

GPU textures/buffers/pipelines keyed by content/revision and device generation. Device loss invalidates GPU cache only. Recreate without modifying document.

# Frame scheduling

Input/animation/viewport changes request frames; idle app does not spin continuously. Visible interactive work has priority over offscreen thumbnails. Minimize redundant uploads and intermediate surfaces.

# Partial updates

Scene extraction consumes ChangeSets and emits scene deltas where beneficial. Full rebuild remains correctness fallback and test oracle.

# Color boundary

RenderScene preserves semantic color/context enough for display transform; monitor RGB conversion is late. Proofing is a render context.

# Overlays

Selection bounds, nodes, guides, snapping, cursors and measurements are non-document overlays with independent invalidation.

# Tests/perf

CPU/reference render where possible, deterministic structural scene snapshots, GPU visual goldens with tolerance, resize/HiDPI/device-loss tests, large scene budgets and intermediate-surface memory telemetry.

# RenderScene ownership and snapshot model

`RenderScene` is immutable from renderer perspective for the duration of a frame/evaluation generation. Scene extraction reads a committed document revision plus optional explicit preview-transaction overlay and produces backend-neutral scene records keyed by stable semantic IDs.

Recommended conceptual hierarchy:

```
RenderScene
├── scene_revision / document_revision
├── surfaces
├── draw_items (stable DrawItemId)
├── resources (images/glyphs/gradients/masks)
├── clip/isolation graph
├── color/proof context
└── overlay_scene (separate editor-only channel)
```

Render backends never acquire mutable access to document objects.

# Draw order and compositing order

Sibling order comes from the evaluated document hierarchy. For each group/object, compositor semantics are explicit and backend-independent:

1. resolve source/evaluated child content;
2. apply local object transform;
3. resolve fill/stroke/text/raster fragments;
4. apply local effects that semantically occur before group compositing;
5. apply clip/mask coverage in the documented order;
6. composite children into isolated intermediate surface when isolation/effect/blend requires it;
7. apply object/group opacity and blend mode against parent backdrop according to the blend specification;
8. apply parent clip/mask/isolation recursively.

The exact effect order is defined by EffectChain/Appearance contracts; renderer code must not reorder for convenience unless mathematically proven equivalent.

# Blend-mode reference contract

Define blend modes once in a renderer-independent spec with normalized channel/alpha equations or normative reference fixtures. At minimum V1-supported blend modes must have:

- straight/premultiplied alpha boundary definition;
- separable/non-separable classification;
- color-space in which blending occurs;
- behavior for alpha=0/1 edge cases;
- reference CPU implementation or golden fixtures.

Vello/wgpu are implementations of this contract, not the source of truth.

# Intermediate surface allocation

The compositor uses offscreen/intermediate surfaces only when required by isolation, filters, masks, nontrivial blend/backdrop semantics, export quality or backend limitation. Allocation planner tracks:

- required pixel bounds + padding from effects;
- sample/format/color-space requirements;
- reuse compatibility;
- lifetime within frame/pass graph;
- memory weight.

Oversized effects/objects are tiled or clipped to safe bounds rather than allocating an unbounded texture from malicious/accidental geometry.

# Surface formats

Internal GPU formats are backend details but must preserve required precision. Display path may use a performant linear/sRGB-capable format; Photo/16-bit/effect paths must not force canonical 16-bit data through an irreversible 8-bit intermediate when fidelity matters. Any precision-reducing preview path is labeled `InteractivePreview` and final/export evaluation uses the required high-fidelity path.

# Color pipeline placement

Canonical/evaluated colors enter render scene in a tagged working representation. Compositing/effects happen in the declared working/blend color context. Proof/display transforms occur late, after scene compositing where the color contract requires, then output to swapchain/display encoding.

Canvas UI theme colors/editor overlays use UI/design-system color semantics and are composited in the editor-overlay pass; they do not inherit document proof/CMYK transforms unless an overlay specifically visualizes document color.

# Vello/wgpu boundary

Vello is responsible for vector/glyph rasterization/rendering it supports efficiently. Aubrieta's compositor owns cross-domain ordering and final convergence. Do not require raster Photo effects to masquerade as Vello vector constructs. When Vello output feeds wgpu composition, resource ownership/synchronization is wrapped behind `aubrieta_vector_renderer`/`aubrieta_compositor` adapters.

# Scene delta contract

Scene extraction may publish `RenderSceneDelta` with created/removed/changed draw items/resources and dirty bounds. Delta application is an optimization; every backend must support rebuilding from a full scene snapshot for correctness recovery/tests.

A delta carries base scene generation. If backend generation does not match, reject the delta and request/rebuild full scene rather than applying to wrong state.

# Frame request/coalescing

Frame scheduler coalesces redundant invalidations but never drops the latest semantic state. Sources include viewport/input, preview transaction, scene delta, animation/motion, background evaluator completion, theme/UI overlay change and window resize. Idle windows do not request continuous frames solely for polling.

Input-responsive frame work is budgeted before low-priority scene/background uploads. Expensive upload preparation can occur off-thread but GPU submission follows backend ownership constraints.

# GPU resource keying

Every GPU resource key includes device generation + semantic content key. Examples:

- raster tile: device_generation, PixelLayerId, raster_revision, tile_coord, representation;
- image: device_generation, ResourceId, content_fingerprint, decoded_variant;
- glyph atlas entry: device_generation, font_face fingerprint, glyph id, render parameters;
- gradient/pattern: device_generation, resource/evaluation fingerprint.

Device loss makes all previous-generation handles invalid automatically.

# Device lifecycle state machine

```
Uninitialized → Ready
Ready → Lost/Outdated
Lost → Recreating → Ready
Recreating → FailedRecoverable | Ready
```

During loss/recreation:

- canonical document stays open/editable where feasible;
- renderer stops dereferencing old handles;
- in-flight GPU-derived results are discarded;
- UI exposes recovery state only if noticeable/blocking;
- caches lazily repopulate from canonical CPU/evaluated data;
- repeated failure can fall back to reduced/headless-safe mode where product policy allows rather than corrupting document.

# Window/swapchain changes

Resize, minimize, surface-outdated/lost and mixed-DPI transitions are normal states. Zero-sized/minimized surfaces suspend presentation without destroying canonical renderer state. Swapchain recreation is isolated from document/session lifecycle.

# Overlay contract

Editor overlays have explicit z-order classes, for example:

1. document content;
2. proof/gamut visualization tied to document;
3. selection/object outlines;
4. guides/grid/snapping/measurements;
5. active tool handles/HUD anchors;
6. cursor-specific visual aids.

Accessibility/high-contrast overlays may replace colors/styles through UI tokens without changing document renderer state.

# Tiled/large-scene rendering

Very large Surfaces/effects/images must support viewport culling and bounded intermediate allocations. Scene extraction uses spatial index/bounds to avoid submitting invisible items when safe. Culling is conservative: false positives cost performance; false negatives are correctness bugs.

# Error/fallback behavior

A single failed shader/pipeline/effect/resource decode produces a scoped diagnostic and visible fallback where possible (placeholder, bypassed effect with warning, CPU/reference path) rather than crashing whole renderer. Fallback must never silently change exported final output; export fails/preflights if required fidelity cannot be produced.

# Performance telemetry

Per frame/developer diagnostics track at least:

- CPU scene extraction time;
- evaluator wait time;
- GPU upload bytes/count;
- vector pass time;
- raster/effect pass time;
- compositor pass/intermediate count;
- present time;
- GPU memory estimate by class;
- culled vs submitted draw items;
- full rebuild vs delta frames;
- device-recovery events.

# Required gauntlets

- exact ordering of nested opacity/blend/mask/clip/isolation fixtures;
- vector+raster+text group with effects across Design↔Photo switching;
- 16-bit raster through interactive and final-quality paths;
- 10k/100k scene-object synthetic culling stress;
- huge offscreen object/effect refusing unbounded intermediate allocation;
- repeated resize/minimize/mixed-DPI/surface recreation;
- forced GPU device loss during idle, pan, paint preview and export preview;
- full-scene rebuild equivalence against arbitrary delta sequences;
- proof/display transform changes invalidating only appropriate render layers;
- overlay readability over white/black/high-frequency artwork.