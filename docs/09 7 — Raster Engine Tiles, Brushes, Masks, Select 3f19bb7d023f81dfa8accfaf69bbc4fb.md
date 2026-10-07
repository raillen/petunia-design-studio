# 09.7 — Raster Engine: Tiles, Brushes, Masks, Selections & Pixel Storage

# Tile model

Raster images stored in independently addressable tiles/chunks with configurable baseline 256×256. Canonical pixel format retains document semantics/bit depth; display texture is derived.

# Storage

TileStore supports resident RAM, compressed backing and package resource. Dirty tiles tracked by revision. Sparse transparent tiles need not allocate.

# Brush engine

BrushStrokeSession receives normalized sample batches. Stages: resample -> stabilize -> dynamics -> dab generator -> mask/texture -> blend -> damage. Commit records touched tile deltas/reference strategy.

# Pixel formats

Baseline 8/16-bit integer and float where scope requires; premultiplied internal compositing policy documented. Color conversions explicit through ColorEngine.

# Selection

PixelSelection tile mask with combine, feather and derived edge representation. Marching ants is render overlay, not selection truth.

# Masks

Pixel masks share tile infrastructure; vector masks evaluate vector path into mask/render stage; compound masks combine nodes.

# Mipmaps

Generated derived levels for zoom/performance; never modify full-res source. Invalidation propagates from dirty base tiles.

# Large images

Decode/import streams/chunks where library allows; avoid full duplicate buffers. Memory-mapped/temp backing is platform adapter with safe cleanup.

# Undo

Brush transactions store changed tile regions/deltas or copy-on-write snapshots according to benchmarked policy.

# Tests

Golden blend modes, brush determinism with fixed seed, pressure curves, tile-edge strokes, huge sparse doc, undo/redo, crash during backing write.

[09.7.1 — Canonical Pixel Formats, Alpha, Channel Layout, Bit Depth & Color Semantics](09%207%201%20%E2%80%94%20Canonical%20Pixel%20Formats,%20Alpha,%20Channel%20L%203f19bb7d023f817993daf2690a90f738.md)

[09.7.2 — TileStore: Coordinates, Sparse Allocation, COW, RAM/Disk Backing & Cache Eviction](09%207%202%20%E2%80%94%20TileStore%20Coordinates,%20Sparse%20Allocation,%203f19bb7d023f8130ba57f5123e9654f0.md)

[09.7.3 — Brush Pipeline: Sampling, Resampling, Stabilizers, Dynamics, Dabs, Blending & Determinism](09%207%203%20%E2%80%94%20Brush%20Pipeline%20Sampling,%20Resampling,%20Stab%203f19bb7d023f81e8b8a5c10a769ee7b2.md)

[09.7.4 — Pixel Selection & Mask Engine: Tile Masks, Feathering, Morphology, Refine & Marching Ants](09%207%204%20%E2%80%94%20Pixel%20Selection%20&%20Mask%20Engine%20Tile%20Masks,%203f19bb7d023f811eaefde94f67f211cf.md)

[09.7.5 — Raster Undo, Dirty Regions, Mips, GPU Residency & Huge-Document Behavior](09%207%205%20%E2%80%94%20Raster%20Undo,%20Dirty%20Regions,%20Mips,%20GPU%20Res%203f19bb7d023f81148236c0f048c51390.md)

[09.7.1 — Canonical Pixel Formats, Alpha Semantics, Layout & Working Precision](09%207%201%20%E2%80%94%20Canonical%20Pixel%20Formats,%20Alpha%20Semantics,%203f19bb7d023f8100babcf0036bae4831.md)

[09.7.2 — TileStore Coordinates, Sparse Storage, Copy-on-Write, Residency & Eviction](09%207%202%20%E2%80%94%20TileStore%20Coordinates,%20Sparse%20Storage,%20Co%203f19bb7d023f81d48ef1cd0c96d09da9.md)

[09.7.3 — Brush Stroke Pipeline, Sample Resampling, Dynamics, Dabs & Determinism](09%207%203%20%E2%80%94%20Brush%20Stroke%20Pipeline,%20Sample%20Resampling,%203f19bb7d023f81f6ba68dd9439bb6505.md)

[09.7.4 — Raster Undo Delta Model, Snapshots, Compression & History Memory Budget](09%207%204%20%E2%80%94%20Raster%20Undo%20Delta%20Model,%20Snapshots,%20Compr%203f19bb7d023f81a7b0d5e0ae17ffb4c1.md)

[09.7.5 — Masks, Pixel Selection, Feathering, Morphology & Mip/Preview Semantics](09%207%205%20%E2%80%94%20Masks,%20Pixel%20Selection,%20Feathering,%20Morph%203f19bb7d023f81c3bc14c2d48e91ed06.md)