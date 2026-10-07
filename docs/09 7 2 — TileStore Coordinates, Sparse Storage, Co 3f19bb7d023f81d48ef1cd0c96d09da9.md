# 09.7.2 — TileStore Coordinates, Sparse Storage, Copy-on-Write, Residency & Eviction

# Tile coordinates

Raster extent maps document-local pixel integer coordinates to TileCoord{x,y}. Tile size is format metadata, initially 256×256 benchmark baseline. Negative document placement occurs via object transform, not negative tile array indexes unless infinite-canvas raster mode is explicitly introduced.

# Tile states

AbsentTransparent, ResidentClean, ResidentDirty, CompressedResident, BackedOnDisk, Loading, Error/Corrupt.

# Sparse

An absent tile represents fully transparent/zero canonical value and consumes no pixel allocation. Creating first nonzero edit materializes tile.

# Ownership

RasterLayer references immutable/versioned TileSet state. Editing transaction acquires writable tile through copy-on-write if snapshots/history/readers share backing.

# Generation

Each tile has content revision/generation. Derived mip/GPU resources reference generation and are invalid if base generation changes.

# Residency hierarchy

1. GPU residency derived;
2. uncompressed RAM;
3. compressed RAM optional;
4. mmap/temp/backing;
5. package resource.

Eviction policy respects dirty state: dirty canonical data must be durably staged before RAM eviction.

# Locking

Tile-level locks or scheduler ownership prevent simultaneous conflicting writes. Read jobs consume immutable buffer/version. Never hold document-global mutex during codec/filter work.

# Prefetch

Viewport predicts near-visible tiles. Priority visible > near viewport > analysis/export background.

# Memory budgets

Configurable hard/soft budgets per RAM/VRAM. Evict derived before canonical resident buffers. Diagnostic exposes largest layers/cache pressure.

# Tests

Sparse 1M×1M conceptual canvas, random writes, COW snapshot isolation, eviction under memory pressure, stale GPU tile invalidation, concurrent readers/writer and backing corruption.