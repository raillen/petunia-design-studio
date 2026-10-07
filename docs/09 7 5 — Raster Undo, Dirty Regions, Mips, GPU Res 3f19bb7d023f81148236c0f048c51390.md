# 09.7.5 — Raster Undo, Dirty Regions, Mips, GPU Residency & Huge-Document Behavior

# Dirty region

Each tile mutation reports minimal dirty RectI plus tile revision. Renderer can upload subregion when backend efficient; otherwise whole tile.

# Undo payload

Brush/local edits: before-tile COW reference + optional compressed changed rect when benchmark wins.

Large filter: old tile-set references retained; new tiles staged then atomically swapped.

Resize/resample: new raster resource staged; old resource retained by history reference.

# Compression

Cold undo tile payload may compress asynchronously after commit; history entry remains usable during compression via original reference. Compression cannot block UI thread.

# Mips

Mip generation is derived job from tile revisions. Each mip tile fingerprint includes source revision set. Stale jobs discarded.

# GPU residency

GPU tile cache tracks CPU source revision. Upload only changed revision. GPU eviction does not affect canonical/undo. Device loss invalidates entire GPU residency cleanly.

# Huge docs

Viewport loads visible full-res/mip tiles only. Operations with global dependency stream tile bands or stage external backing. Refuse impossible allocation early with estimated memory/disk diagnostics.

# Memory pressure

OS/app pressure lowers background caches before interactive/source/history data. Hard failure returns recoverable MemoryPressureError rather than process OOM when predictable.

# Benchmarks

8k/16k photos, gigapixel sparse canvas, thousands of brush strokes, 500-layer composites, undo depth under budget, pan/zoom residency churn.