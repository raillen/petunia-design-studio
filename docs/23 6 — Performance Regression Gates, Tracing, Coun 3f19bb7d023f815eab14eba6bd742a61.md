# 23.6 — Performance Regression Gates, Tracing, Counter Set & Optimization Protocol

# Protected metrics

Frame CPU/GPU, input latency, hit/snap, brush backlog, open/save, render/export throughput, RSS/VRAM, cache hit/miss, allocations, text layout and model-view updates.

# Regression threshold

Each metric has runner-calibrated noise band. Beyond threshold CI fails or flags review. Waiver records cause, benefit and expiry/revisit trigger.

# Trace correlation

ActionId/ToolId, document revision, JobId and render frame ID correlate Python events, native spans and GPU markers.

# Counter set

Draw calls/passes, tessellated vertices, texture uploads, tile reads/writes, shader/pipeline misses, text glyph cache, geometry index queries, cache evictions, Python↔native calls/frame.

# Optimization protocol

Reproduce -> profile -> hypothesis -> correctness oracle -> optimize -> benchmark -> inspect counters/traces -> document result. No optimization accepted solely from code intuition.

# Performance debt

Known slow paths enter explicit ledger with fixture, severity, owner, workaround and target milestone.