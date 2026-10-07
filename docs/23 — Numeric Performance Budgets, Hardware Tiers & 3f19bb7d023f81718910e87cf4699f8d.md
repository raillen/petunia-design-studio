# 23 — Numeric Performance Budgets, Hardware Tiers & Release SLOs

# Purpose

Substituir adjetivos como “rápido” por budgets mensuráveis.

# Initial tiers

Tier L: 4-core modern x86/ARM, 8 GB RAM, integrated GPU, 1080p.

Tier R: 8-core, 16 GB, mainstream discrete/integrated modern GPU, 1440p.

Tier H: 12+ core, 32 GB, discrete GPU, 4K.

Exact reference machines are recorded by benchmark lab.

# Interactive initial targets on Tier R

Canvas pan/zoom normal fixture: p50 <= 8 ms, p99 <= 16.7 ms frame CPU+GPU target.

Pointer-to-transform preview: p50 <= 8 ms, p99 <= 16.7 ms.

Brush input-to-visible dab: p50 <= 12 ms, p99 <= 24 ms.

Hit-test 100k simple objects: p95 <= 4 ms after spatial index warm.

Snap query typical 10k visible candidates/indexed: p95 <= 4 ms.

Layers scroll 10k rows: 60 Hz target, no full model reset.

# Responsiveness

GUI-thread task >8 ms logged in dev; >50 ms considered stall. Background progress updates <=20 Hz normally.

# Memory

Idle app target <=500 MB Tier R before large docs; budgets refined after real backend. Cache must respect configured RAM/VRAM ceilings and shed before OS pressure causes thrash.

# Startup/open

Cold launch to interactive shell target <=3 s Tier R SSD; warm <=1.5 s goal.

PTND progressive open should expose first usable view before all offscreen resources load.

# Rule

Numbers are initial engineering SLOs, not marketing promises; every deviation requires fixture/hardware evidence and explicit revision.

[23.1 — Hardware Reference Tiers, Benchmark Lab & Reproducibility](23%201%20%E2%80%94%20Hardware%20Reference%20Tiers,%20Benchmark%20Lab%20&%20R%203f19bb7d023f8117b05bfa9c3ae2d15b.md)

[23.2 — Interactive Latency SLOs: Canvas, Input, Tools, Snapping & Panels](23%202%20%E2%80%94%20Interactive%20Latency%20SLOs%20Canvas,%20Input,%20Too%203f19bb7d023f81588bb5efdb9499b8f1.md)

[23.3 — Document Scale SLOs: Objects, Paths, Text, Raster, Surfaces & Libraries](23%203%20%E2%80%94%20Document%20Scale%20SLOs%20Objects,%20Paths,%20Text,%20R%203f19bb7d023f81358706f199e50e9944.md)

[23.4 — Memory, Cache, VRAM, History & Long-Session SLOs](23%204%20%E2%80%94%20Memory,%20Cache,%20VRAM,%20History%20&%20Long-Session%203f19bb7d023f81798050e49ea0b32be6.md)

[23.5 — Startup, Open/Save, Import/Export & Background Throughput SLOs](23%205%20%E2%80%94%20Startup,%20Open%20Save,%20Import%20Export%20&%20Backgro%203f19bb7d023f81248006f62c1c76243a.md)

[23.6 — Performance Regression Gates, Tracing, Counter Set & Optimization Protocol](23%206%20%E2%80%94%20Performance%20Regression%20Gates,%20Tracing,%20Coun%203f19bb7d023f815eab14eba6bd742a61.md)