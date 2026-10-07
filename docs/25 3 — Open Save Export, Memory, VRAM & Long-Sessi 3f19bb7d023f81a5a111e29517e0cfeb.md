# 25.3 — Open/Save/Export, Memory, VRAM & Long-Session Budgets

# Fixtures

Small 10 MB, Medium 250 MB, Large 1 GB PTND; vector-heavy 100k objects; raster 8k layers/images; mixed poster; long text; 100-page light layout.

# Provisional recommended-tier

Cold app launch to interactive shell target ≤ 2.5 s excluding OS security first-run.

Small PTND first usable view ≤ 1 s.

Medium first usable view ≤ 3 s with lazy resources.

Save should stream/background with GUI stall ≤ 16 ms; total throughput target set after codec benchmark.

PNG 4K export target measured/regression locked rather than arbitrary quality sacrifice.

PDF vector export 100-page fixture gets explicit throughput after implementation.

# Memory

Idle shell target ≤ 500 MB provisional including Python/Qt/native runtime.

Opening resource should avoid >1.5×–2× full decoded image duplication unless operation demands.

Cache obeys configured soft budget and returns memory after document close within bounded idle window.

# Long session

8-hour scripted edit/open/close workload: no unbounded object/cache/job growth; steady-state leak budget effectively zero, residual allocator/cache growth documented.

# VRAM

Renderer exposes budget/usage and evicts derived resources before device failure. Large document must degrade/reload rather than crash from predictable budget pressure.