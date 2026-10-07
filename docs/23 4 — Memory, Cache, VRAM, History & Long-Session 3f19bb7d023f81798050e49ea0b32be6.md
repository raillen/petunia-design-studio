# 23.4 — Memory, Cache, VRAM, History & Long-Session SLOs

# Budgets

Per-process memory budgets are configurable based on physical RAM. Cache manager reserves headroom for OS/application, never attempts to consume all free memory.

# Initial Tier R goals

Idle shell <=500 MB RSS goal pending backend measurements.

Native/document overhead for simple vector doc should be dominated by actual content, not per-object megabyte-scale metadata.

GPU cache has configurable cap/default based on reported budget; device memory allocation failures trigger eviction/fallback, not crash.

# History

History soft budget defaults as fraction/absolute cap; cold raster history compresses/spills. UI can expose current undo memory and reduced depth only when material.

# Long session

8-hour synthetic session: repeated open/edit/close, tool switching, exports, plugin restarts. Memory after cleanup should converge near bounded caches; unexplained monotonic growth is release blocker.

# Leak evidence

ASan/LSan/platform tools + app-level counters. Qt QObject/widget counts and native resource counts sampled at document close.

# Pressure

Under OS memory pressure, derived caches shed first; canonical document/history durability policies remain safe.

# Tests

Huge image, 100 docs sequentially, GPU device loss, history pressure, thumbnail cache and plugin process crash/restart.