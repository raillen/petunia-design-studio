# 14.2.3 — Open, Save, Export, Geometry, Raster & Text Throughput Budgets

# Initial target fixtures

Small 10 MB PTND; Medium 250 MB mixed; Large 1 GB+ sparse/linked; vector stress 100k objects; raster 8k/16k; brochure 20 pages.

# Startup

Cold app to usable empty shell target <= 2.5 s Tier R, <= 5 s Tier L.

Warm startup <= 1.5 s Tier R.

# Open

Small PTND first usable frame <= 1 s.

Medium PTND first usable viewport <= 3 s Tier R via lazy resources.

Large doc should show shell/progress/partial viewport rather than waiting for all resources.

# Save

Incremental/snapshot save 250 MB mixed target <= 5 s Tier R when storage allows; UI remains editable if snapshot model supports.

Autosave must not block GUI event loop > 4 ms contiguous under normal fixture.

# Export

4K PNG typical mixed poster target <= 2 s Tier R.

20-page vector/text PDF target tracked with per-page throughput and must remain cancellable.

Batch 100 simple social Surfaces measures outputs/min rather than arbitrary one target.

# Geometry

Boolean 100 simple shapes <= 50 ms typical; 10k-complexity operation can background.

Hit/spatial index rebuild incremental; full 100k objects benchmark budget set from measured baseline.

# Raster

8k Gaussian blur radius 20 target GPU interactive preview <= 100 ms/update after warm pipeline; final may be slower but asynchronous.

Brush 1 minute continuous stroke must not grow queue/memory unbounded.

# Text

20-page story initial layout target <= 300 ms Tier R after fonts cached; incremental paragraph edit much lower.

# Rule

Budgets are tuned after first implementation baseline; regressions > agreed threshold require evidence/waiver.