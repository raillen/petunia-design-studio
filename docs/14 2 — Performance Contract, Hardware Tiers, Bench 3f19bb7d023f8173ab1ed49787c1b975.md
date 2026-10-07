# 14.2 — Performance Contract, Hardware Tiers, Benchmarks & Long-Session Budgets

# Hardware tiers

Define Low/Recommended/High reference machines per OS including CPU, RAM, GPU, VRAM, resolution/DPI. Results always name tier.

# Interactive metrics

Pointer-to-preview p50/p99, frame time, dropped frames, snap/hit-test latency, brush sample backlog, text edit latency, panel/model update.

# Operations

Open/save PTND sizes, image import, SVG/PDF import/export, boolean complexity, text layout, filter preview/final, data merge throughput.

# Memory

Idle baseline, per-document, tile cache, derived caches, VRAM, plugin process, recovery journal. Long-session test opens/edits/closes docs repeatedly to detect growth.

# Profiling

C++: Tracy/perf/VTune/Instruments/ETW + GPU tools. Python: py-spy/cProfile/scalene where suitable. Qt event-loop stalls instrumented.

# Regression gates

Statistical medians/percentiles with noise threshold on pinned runner. Significant regression fails or requires reviewed waiver with cause.

# Optimization rule

Profile -> hypothesis -> change -> correctness -> before/after benchmark -> profiler/counter verification. No speculative SIMD/cache rewrite.

[14.2.1 — Reference Hardware Tiers & Benchmark Environment](14%202%201%20%E2%80%94%20Reference%20Hardware%20Tiers%20&%20Benchmark%20Envi%203f19bb7d023f812fa439f0554b65de58.md)

[14.2.2 — Interactive Latency & Frame-Time Budgets](14%202%202%20%E2%80%94%20Interactive%20Latency%20&%20Frame-Time%20Budgets%203f19bb7d023f810297e2e30626d599a6.md)

[14.2.3 — Open, Save, Export, Geometry, Raster & Text Throughput Budgets](14%202%203%20%E2%80%94%20Open,%20Save,%20Export,%20Geometry,%20Raster%20&%20Te%203f19bb7d023f8164b391c0bea154793e.md)

[14.2.4 — RAM, VRAM, Cache, History & Long-Session Budgets](14%202%204%20%E2%80%94%20RAM,%20VRAM,%20Cache,%20History%20&%20Long-Session%20%203f19bb7d023f817889fcdbc6a517f0d7.md)

[14.2.5 — Benchmark Methodology, Statistics, Regression Thresholds & Evidence Format](14%202%205%20%E2%80%94%20Benchmark%20Methodology,%20Statistics,%20Regres%203f19bb7d023f81328d08c9dd78793302.md)