# 14.2.5 — Benchmark Methodology, Statistics, Regression Thresholds & Evidence Format

# Benchmark classes

Microbench: geometry kernels, tile codec, color transforms.

Subsystem: brush stroke, text layout, scene build.

Scenario: actual workflows in golden documents.

UI latency: input-to-visible markers.

Long-run: leaks/cache growth.

# Repetitions

Warmups excluded. Run enough iterations for stable distribution; report median, p95/p99 where latency matters, standard deviation/MAD and sample count.

# Cold/warm

Open/save/import benchmarks explicitly label OS cache cold-ish/warm, Petunia cache cold/warm and GPU pipeline warm state.

# Regression threshold

Default CI signal: >10% regression beyond noise on stable pinned runner; stricter for latency-critical budgets, looser for noisy end-to-end. Threshold established per benchmark variance.

# Correctness first

Performance result invalid if output/golden changed unexpectedly. Optimization PR includes correctness suite before timing comparison.

# Evidence JSON

benchmarkId, fixtureVersion, commit/build, environment, samples, summary stats, counters, profiler artifact references and comparison baseline.

# Profiling

A benchmark regression should trigger profiler/counter analysis before “optimizing.” GPU/CPU split captured where renderer involved.

# Publishing

Performance claims in docs/release use reproducible benchmark ID and hardware tier, never unqualified “X times faster.”