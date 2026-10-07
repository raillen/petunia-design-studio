# 25.4 — Performance Regression Thresholds, Profiling Evidence & CI Policy

# Threshold

Microbenchmarks: regression >5–10% with statistical confidence triggers review depending noise.

Interactive end-to-end: p95/p99 degradation crossing SLO is blocker unless approved waiver.

Memory: >10% baseline increase on fixed fixture requires explanation.

# Evidence

Before/after benchmark, repetitions, environment, raw data, profiler trace/counters and correctness tests.

# CI

Fast smoke benchmarks every PR; stable dedicated runner full suite nightly/release. Do not fail noisy shared runners on tiny deltas.

# Profiling

Performance agent identifies dominant cost. Optimization accepted only if bottleneck evidence moves and output correctness unchanged.

# Waiver

Time-bounded waiver records cause, owner, follow-up Goal and user impact. SLO cannot silently drift upward release after release.