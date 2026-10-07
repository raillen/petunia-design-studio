# 14.2.4 — RAM, VRAM, Cache, History & Long-Session Budgets

# Idle

Initial engineering goal: empty app resident memory <= 500 MB Tier R including bundled Python/Qt/native renderer. Measure and tighten after real baseline; no hidden multi-GB idle.

# Document memory

Canonical vector/text metadata should scale approximately O(objects/runs), not duplicate render structures per view.

Raster decoded RAM governed cache budget rather than full-document size.

# Default cache fractions

Use adaptive budgets, e.g. CPU raster/derived cache soft cap <= min(configured MB, ~25% available RAM) and GPU cache <= conservative fraction of reported budget. Exact platform formula ADR after telemetry/benchmarks.

# History

Default soft history budget initial 1–2 GB on 16 GB system but adaptive to available memory; cold raster undo payload can compress/spill. Never consume all RAM to preserve arbitrary undo depth.

# VRAM

Visible/near-view textures, atlases and transient frame targets prioritized. Renderer tracks peak transient target requirement before effect; avoids single operation exhausting device when tiling possible.

# Long session

8-hour scripted session with open/edit/close cycles must show no monotonic unexplained growth beyond bounded caches.

100 document open/close cycles returns near stabilized baseline after caches trim.

# Plugin processes

Per-plugin memory quota/default warning threshold separately tracked; plugin leak cannot consume main process unchecked.

# Memory pressure

Test injected low-memory signal and explicit cache trim. App should degrade cache/perf, not lose canonical edits.

# Leak evidence

ASan/LSan where supported, heap snapshots and GPU validation/resource counters.