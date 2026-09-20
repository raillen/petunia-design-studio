# 14.2 — Performance Contract, Hardware Tiers, Benchmark Corpus & Regression Gates

<aside>
⚡

Performance is a measured behavior, not an adjective. Aubrieta optimizes after correctness, with explicit budgets and representative hardware.

</aside>

# Hardware tiers

**Tier L — Modest:** low-end/older supported CPU/GPU, limited RAM, representative integrated graphics.

**Tier M — Typical:** mainstream current laptop/desktop.

**Tier H — Workstation:** high-end hardware used to expose scalability ceilings, not to excuse Tier L regressions.

Exact reference machines/configurations are versioned once hardware is selected.

# Benchmark record

Every BenchmarkId records scenario, fixture, hardware tier, build profile, warmup, repetitions, metric, median/P95/P99 when meaningful, memory peak, warning threshold, failure threshold and historical trend.

# Initial benchmark families

- AUB-PERF-STARTUP-*;
- AUB-PERF-DOC-OPEN/SAVE/AUTOSAVE-*;
- AUB-PERF-CANVAS-PAN/ZOOM-*;
- AUB-PERF-LAYERS-10K/100K-*;
- AUB-PERF-VECTOR-NODES/BOOLEAN/STROKE-*;
- AUB-PERF-RASTER-BRUSH/TILES-*;
- AUB-PERF-TEXT-SHAPING/REFLOW-*;
- AUB-PERF-EXPORT-*;
- AUB-PERF-PLUGIN-*;
- AUB-PERF-MCP-*.

# UX latency classes

Pointer/pen preview is frame-bound.

Frequent direct manipulation should remain perceptually immediate.

Operations whose delay becomes noticeable expose feedback.

Long jobs expose progress and cancellation when consistency permits.

Numerical thresholds are validated by benchmark evidence rather than frozen from intuition.

# UI budgets

Measure frame time/P95, input-to-visual latency, Layers scroll, panel resize while canvas moves, command palette search, font search, docking drag, histogram interference and mixed-DPI behavior.

# Memory

Track resident memory and subsystem allocations for representative fixtures. Derived caches have explicit eviction/rebuild policies.

# GPU

Measure upload pressure, resource churn, device-loss recovery and fallback paths. Benchmark builds may enable GPUI/Vello/wgpu profiling hooks without enabling them in normal release builds.

# Regression gate

Compare against a versioned baseline on the same tier/config. Large regressions block unless an ADR accepts the tradeoff with evidence.

# Optimization discipline

No unsafe/complex optimization solely from intuition. Profile first; preserve readability and correctness unless measured evidence justifies complexity.

# Evidence

Benchmark raw output and summarized trend live in release evidence. Marketing/user-facing hardware claims require validated Tier L results.