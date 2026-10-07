# 09.7.2 — TileStore: Coordinates, Sparse Allocation, COW, RAM/Disk Backing & Cache Eviction

# Tile address

TileKey = RasterResourceId/LayerId + mip level + signed tileX/tileY + plane/channel variant if applicable. Negative document coordinates are valid and floor-division mapping is specified.

# Tile geometry

Baseline 256×256 pixels, benchmark adjustable before format freeze. Edge tiles may be partial logically but storage can remain full tile for simplicity; valid rect metadata identifies meaningful pixels.

# States

UnallocatedTransparent, ResidentClean, ResidentDirty, CompressedResident, BackedOnDisk, Loading, Error/Corrupt.

# Sparse semantics

Absent tile reads as transparent/zero according pixel model and does not allocate. Writing first nonzero region materializes tile.

# COW

TilePayload immutable when shared by history/snapshots. Mutable edit requests unique writable payload; clone occurs lazily. Revision increments per tile content mutation.

# Backing

Canonical PTND resource, recovery/temp backing and cache backing are distinct origins. Dirty canonical tiles cannot be evicted unless safely represented in document/history/recovery state.

# Cache

Separate budgets for decoded CPU tiles, compressed tiles, mip tiles and GPU residency. Priority visible viewport > active tool neighborhood > near viewport > background. LRU/clock hybrid allowed behind policy.

# Locking

Tile access uses scoped read/write handles. Never hold tile locks while invoking arbitrary callbacks or long GPU operations. Multi-tile operations acquire deterministic ordering or snapshot data to avoid deadlocks.

# Prefetch

Viewport predicts neighboring tiles based on pan/zoom velocity. Prefetch jobs cancellable and lower priority than interactive requests.

# Diagnostics

Resident/compressed/disk bytes, hit/miss, allocation count, COW clones, eviction, load latency.

# Tests

Negative coords, sparse huge canvas, COW history isolation, memory pressure, concurrent reads, deterministic locking, corruption and recovery.