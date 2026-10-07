# 09.7.4 — Raster Undo Delta Model, Snapshots, Compression & History Memory Budget

# Goal

Undo raster edits without copying entire high-resolution layer per stroke.

# Tile delta strategy

For each touched tile capture pre-edit generation/data lazily on first write in transaction. Post-edit data is canonical current tile. Undo swaps/restores prior tile version; redo can restore retained post version or replay stored post tile.

# COW snapshot

Tile buffers are immutable blobs referenced by shared internal handles. Edit creates new buffer only for touched tile. History entry retains old handle; document retains new handle.

# Compression

Cold historical tile blobs may compress asynchronously with lossless codec. History manager never blocks pointer-up on full compression; retention transitions must preserve availability.

# Memory budget

History owns soft/hard byte budgets. Eviction candidates oldest undo entries beyond save/checkpoint policy. Before dropping history, UI/history reports truncation boundary if user-visible.

# Huge operation

If one operation touches enormous percentage of image, choose between per-tile COW, checkpoint-backed delta or temporary history store based cost model. Strategy can differ without changing Command semantics.

# Redo

History retains post-version references until branch invalidated. New mutation after undo discards redo branch and releases buffers asynchronously.

# Save point

Explicit save marks revision but does not require clearing history. Persisting history inside PTND is not V1; session history/recovery separate.

# Crash recovery

Recovery journal may record raster changed tile blobs/checkpoints independently from undo storage.

# Tests

Thousands of strokes, undo/redo pixel equality, memory budget eviction, branch invalidation, compressed-history restoration, disk-full recovery backing failure and no UI stall over budget.