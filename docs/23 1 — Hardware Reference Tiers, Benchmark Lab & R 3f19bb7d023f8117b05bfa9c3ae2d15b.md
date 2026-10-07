# 23.1 — Hardware Reference Tiers, Benchmark Lab & Reproducibility

# Reference machines

Every performance claim names OS/build, CPU, RAM, GPU/driver, VRAM/shared memory, display resolution/DPR, storage and power mode.

# Tier L

Modern 4-core class CPU, 8 GB RAM, integrated GPU, SSD, 1080p. Represents minimum commercially usable profile, not obsolete hardware.

# Tier R

Modern 8-core class CPU, 16 GB RAM, mainstream integrated/discrete GPU, SSD, 1440p. Primary engineering SLO tier.

# Tier H

12+ core CPU, 32 GB RAM, discrete GPU, NVMe, 4K/high-DPI. Used for heavy professional workloads, not for hiding regressions.

# Benchmark environment

Fixed fixtures, warm-up policy, power/performance state, background process control, renderer backend, cache cold/warm distinction and build flags.

# Statistics

Report median, p95/p99 where interaction, confidence/noise and sample count. Reject single-run claims.

# Baseline

Each protected benchmark stores accepted baseline per hardware runner; regression threshold accounts for runner variance.

# Artifacts

Raw timings/counters/traces retained for significant regressions and release candidates.