# 09.6 — Raster Engine: Tiles, Brushes, Masks, Selections & Pixel Storage

# Storage

PixelLayer uses sparse/tiled storage instead of assuming one monolithic texture. **Resolved V1 default:** logical CPU tiles are **128×128 px**. This gives substantially finer dirty/undo granularity than 256/512 while still being large enough to batch efficiently. Tile size remains a storage/engine parameter rather than creative document semantics; GPU atlases/pages use their own larger allocation geometry and are not forced to match CPU tiles.

Hot resident tiles remain uncompressed. Cold/swap tiles may use a low-latency compressor such as LZ4; native persisted raster chunks use Zstd or another versioned codec selected by the storage layer. Codec choice is recorded as storage metadata and may change without changing visual semantics.

# Pixel description

PixelFormat + channel model + BitDepth + AlphaMode + ColorProfile. Architecture supports 8/16-bit early; float later. Canonical raster data distinguishes straight/premultiplied alpha contract at boundaries.

# Tile lifecycle

Absent/clean/dirty/resident/compressed/evicted states. CPU canonical tile storage and GPU residency are distinct. Dirty rects inside tiles avoid full-tile work where useful.

# Brush engine

A stroke is sampled from pointer/pen stream with stabilized coordinates, pressure/tilt/velocity inputs, spacing, dynamics, stamp source and blend/paint mode. Preview and commit must be reproducible enough for undo/testing. Brush presets are resources, not code.

# Raster selection

Selection mask can be sparse/tiled grayscale coverage, with combine modes add/subtract/intersect/xor, feather/grow/shrink and transform. Marching ants are derived visualization.

# Masks

Raster/vector masks share compositing semantics but preserve native representation. Mask editing has explicit target to prevent painting wrong content.

# Undo

Raster edits record changed tile deltas/checkpoints with compression and memory budget. Large strokes may stream tile before-images once per transaction.

# GPU/CPU

wgpu compute is acceleration, not canonical semantics. CPU/reference implementation exists for critical operations/testing where feasible.

# Tests

Tile boundaries, huge sparse images, pressure replay, mask composition, cancellation mid-stroke, device loss/reupload, color-profile conversions and memory-pressure eviction.

# V1 pixel-depth contract

**8-bit and 16-bit integer/channel are V1 architectural requirements**, not future placeholders. A PixelLayer/placed raster path must never silently down-convert 16-bit source data merely because a UI preview/backend path is 8-bit. Unsupported operations either:

- execute through a validated conversion policy with explicit user-visible consequence; or
- remain unavailable with a structured reason.

32-bit float remains `POST_V1` unless promoted by ADR.

# Tile address model

Canonical logical tile identity is stable and independent of GPU allocation:

```
TileCoord { x: i32, y: i32 }
TileKey { pixel_layer_id, mip_level=0 for canonical, tile_coord }
```

Canonical editable storage exists only at mip 0. Mip levels, GPU pages and thumbnail pyramids are derived. Sparse negative tile coordinates are allowed if document transforms/painting semantics allow pixel layers to extend beyond an origin.

# Tile payload and edge semantics

A tile stores valid pixel extent, format/depth/profile contract and pixel bytes in a documented row layout. Edge tiles need not allocate a full logical 128×128 payload when storage layer can represent clipped extent safely, but algorithms must behave as if out-of-bounds pixels follow the layer's explicit edge policy rather than reading uninitialized memory.

# Canonical CPU storage vs residency

`PixelLayer` canonical data is CPU/storage-owned. GPU textures are acceleration caches. A dirty GPU tile is not allowed to become the sole newer copy of canonical pixels after a committed operation; commit synchronizes/produces canonical tile data before the transaction is considered durable/undoable.

Interactive brush preview may temporarily live in GPU/working buffers, but transaction commit produces canonical tile deltas and a committed raster revision.

# Tile state machine

```
Absent/ImplicitTransparent
 → ResidentClean
 → ResidentWorkingDirty (active transaction)
 → ResidentCommittedDirty
 → CleanPersisted
 ↔ CompressedCold
 → EvictedDerivedCopy
```

State names can differ in code, but V1 must distinguish canonical committed data, transaction-working data and derived GPU/cache copies. `Absent` means semantic transparent/default tile and must not require material allocation.

# Tile locking/concurrency

Raster jobs operate on immutable input snapshots or explicitly leased tiles. Canonical writes are coordinated by the DocumentSession mutation/transaction owner. Do not expose per-tile mutexes to arbitrary feature code. Multi-tile operations acquire/plan affected tiles in deterministic order to avoid deadlock if internal locking is used.

# Brush stroke pipeline

Canonical stroke lifecycle:

1. pointer/pen events enter timestamped input buffer;
2. input normalization maps coordinates/pressure/tilt/buttons to a stable `BrushInputSample`;
3. optional stabilizer generates a processed trajectory;
4. spacing engine emits brush dabs based on path distance/time policy;
5. dynamics evaluator resolves size/opacity/flow/rotation/etc. from preset + input;
6. dab rasterizer computes coverage/color contribution;
7. paint/blend kernel updates transaction-working tiles;
8. preview invalidation reaches compositor;
9. commit records affected tile before/after delta metadata and advances raster/object revision.

UI frame rate must not change stroke spacing semantics. Event coalescing is allowed only before the deterministic stabilizer/spacing policy in a way covered by replay tests.

# Brush preset schema

Brush presets are versioned resources composed from typed parameters rather than executable arbitrary code. V1 schema must declare defaults, ranges/units, pressure/tilt curves, spacing/stabilization and paint mode. Unknown newer preset fields are preserved when safe. Plugins may provide new brush engines only through explicit capability/type IDs and cannot masquerade as built-in preset parameters.

# Stroke replay and determinism

Store enough transaction data for tests/diagnostics to replay representative strokes: preset fingerprint, normalized input samples or deterministic processed sample stream, engine version and color/selection context references. This is not required to become permanent document history after undo eviction, but reproducibility fixtures must exist.

# Selection mask contract

Canonical selection is session/tool state unless explicitly saved as channel/mask. Coverage is normalized grayscale with a defined range/precision; V1 should use a representation capable of smooth antialias/feather transitions without binary-only loss. Combine operations define exact coverage math and bounds behavior.

Feather/grow/shrink/transform can be derived operations until committed to selection state. Saving selection creates an explicit document mask/channel resource/object according to the functional contract.

# Mask evaluation order

For raster content, effective coverage is derived from object opacity × raster/vector mask evaluation × clip/group semantics as defined by compositor. Editing a mask never bakes it into source pixels unless an explicit Apply/Bake Mask command is invoked.

# Pixel blending semantics

Raster paint/composite kernels consume backend-independent `BlendMode` semantics from the compositor/color contracts. Premultiplication conversion occurs at explicit boundaries. No kernel may reinterpret straight-alpha canonical pixels as premultiplied without typed conversion.

# Color/profile conversions

A PixelSurface owns/points to a color profile/space contract. Painting colors are converted from semantic document/tool color into the target surface working representation through `aubrieta_color`. Profile conversion affecting canonical pixel values is an explicit undoable command; display/proof transforms are derived and never rewrite source pixels.

# Filters and adjustments

Nondestructive filters/adjustments do not rewrite PixelLayer canonical tiles; they evaluate through effect/compositor caches. Destructive filter application, when explicitly requested, is a transaction over dirty tiles with the same undo/storage guarantees as painting.

# Compression and persistence

Compression is storage policy, not pixel semantics. Each persisted raster chunk records codec + codec version/settings needed for decode. Decompression is bounded and checks declared/actual output length to prevent decompression bombs. Corrupt tile/chunk decode yields localized document diagnostics and recovery policy, never unchecked allocation/panic.

# Memory budgets

Track separately:

- canonical resident CPU pixel bytes;
- compressed cold/swap bytes;
- active transaction before-images;
- derived mip/thumbnail bytes;
- GPU residency.

Memory pressure may evict/compress derived/cold data first, but must preserve active transaction correctness. Brush latency budgets take priority over background compression.

# GPU device loss

Device loss discards GPU raster caches and in-flight derived compute. Canonical CPU/storage tiles remain authoritative and are lazily re-uploaded. An active paint transaction must either safely continue through CPU/fresh device path or cancel with exact pre-stroke restoration; it must not commit a partially lost stroke.

# Additional required gauntlets

- paint across 128×128 boundaries at every edge/corner;
- 16-bit round-trip through save/reopen and representative adjustment/filter paths;
- sparse layer spanning very large coordinates with few allocated tiles;
- repeated undo/redo of multi-tile strokes with byte/visual equivalence;
- memory pressure during long stroke and during undo spill;
- corrupt/compression-bomb raster chunks;
- device loss during preview and immediately before commit;
- color-profile conversion with differential/reference checks;
- input replay at different UI frame rates producing equivalent dab placement.