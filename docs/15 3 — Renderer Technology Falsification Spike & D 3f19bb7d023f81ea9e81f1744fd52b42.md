# 15.3 — Renderer Technology Falsification Spike & Decision Gate

# Why spike

Renderer is too foundational for popularity-based choice. Document/render boundary lets us benchmark candidates without locking domain.

# Candidates

- Skia GPU integration;
- Dawn/WebGPU abstraction;
- focused custom renderer over Vulkan/Metal/D3D via suitable abstraction;
- Qt RHI only as candidate adapter, not assumed document engine.

# Workloads

1. 100k simple vector objects;
2. complex Beziers/strokes;
3. 20 artboards mixed vector/raster;
4. 8k raster with pan/zoom;
5. many masks/blend/effects;
6. multilingual text;
7. Gaussian/live filter;
8. high-DPI dual monitor;
9. device loss;
10. headless export.

# Metrics

frame p50/p99, CPU submit, GPU time, memory/VRAM, upload bandwidth, startup/pipeline compile, visual correctness, driver coverage, API integration cost, maintenance.

# Hard requirements

Windows/Linux/macOS, native window embedding, offscreen/headless, color-managed final transform hooks, resource recovery, deterministic tests, acceptable license.

# Decision

Technology Decision Agent produces Pareto matrix + raw evidence. Architect writes ADR. No overall weighted score without explaining hard constraints/trade-offs.

# Escape hatch

RenderScene remains backend-neutral enough to support fallback/testing without creating lowest-common-denominator document semantics.