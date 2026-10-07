# G041 — Canonical Raster Pixel Formats & TileStore

# Goal

Implement tiled canonical raster storage with sparse allocation, COW and memory-safe backing.

# Depends

G003–G008, 09.7.1–09.7.2, PTND tile schema.

# Primary

engine-engineer + systems-architect.

# Deliverables

PixelFormat descriptors; TileKey/state machine; TileStore read/write handles; sparse tiles; COW; RAM/compressed/temp backing; cache budgets/diagnostics.

# Acceptance

Huge sparse canvas opens/edits without full allocation; snapshots/history share safely; negative coords and memory pressure work.

# Tests

8/16/float, alpha, COW, concurrency/TSan, sparse 100k canvas, corruption/backing failure and memory benchmarks.