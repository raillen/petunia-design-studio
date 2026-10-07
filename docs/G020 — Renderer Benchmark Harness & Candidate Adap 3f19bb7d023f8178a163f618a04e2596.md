# G020 — Renderer Benchmark Harness & Candidate Adapters

# Goal

Build neutral benchmark harness capable of comparing renderer candidates before architectural lock.

# Depends

G018, G003, G008.

# Primary

technology-decision-agent + renderer-engineer; performance-agent verifies.

# Skills

rendering-2d, shaders, benchmarking, performance-native.

# Deliverables

Minimal RenderScene IR for benchmark; offscreen/on-screen test harness; candidate adapters for at least Skia GPU, Dawn/WebGPU and feasible Qt/custom candidate; workloads from 15.3; GPU/CPU timing capture; screenshot/golden comparison; memory metrics.

# Acceptance

Same scene inputs execute on candidates with comparable output and machine-readable benchmark results across target OS where candidate supports.

# Evidence

Raw timings, images, driver/environment, integration complexity notes and failures.

# Non-goals

Production renderer decision itself (G021), full text/effects.