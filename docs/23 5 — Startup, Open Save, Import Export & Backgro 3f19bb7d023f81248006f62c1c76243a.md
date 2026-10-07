# 23.5 — Startup, Open/Save, Import/Export & Background Throughput SLOs

# Startup

Tier R cold launch to interactive shell <=3 s initial goal; warm <=1.5 s. Measure separately plugin discovery, font indexing and renderer initialization.

# Progressive open

PTND should render first useful viewport before all offscreen tiles/previews/resources load when package structure allows.

# Save

Save latency measured by canonical bytes/resources and changed-resource incremental opportunities. UI must remain responsive and snapshot revision semantics correct even when total write takes seconds.

# Export

Throughput measured for 4K raster, multi-page PDF/SVG and Data Merge batches. Long export is job with ETA/progress/cancel rather than pretending synchronous instant.

# Import

Header/sniff response quick; large decode streams/background. Time-to-first-visible-image is separate metric from full decode.

# Autosave

Checkpoint work should not create GUI stall >8 ms through main-thread serialization. Native/background serialization snapshot architecture required.

# CI

Throughput regressions use representative fixed documents, cold/warm cache variants and output validation to prevent 'fast but wrong' optimizations.