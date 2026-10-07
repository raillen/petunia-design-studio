# 09.13 — Job System, Concurrency, Cancellation, Priority & Backpressure

# Scheduler

C++ JobScheduler with bounded worker pool, std::jthread/stop_token semantics and priority lanes.

# Priorities

InteractiveCritical, VisibleRender, InteractiveBackground, UserRequested, Maintenance. Background cannot starve pointer/render responsiveness.

# Jobs

JobId, type, owner session, priority, progress model, cancellation, result/error, diagnostics, child jobs.

# Cancellation

Cooperative checks at bounded intervals. Export/import/filter/inpaint must cancel before atomic commit when possible. Cancelled job publishes no stale canonical mutation.

# Backpressure

Bound queue sizes; coalesce replaceable jobs like histogram/thumbnail/render invalidations. Do not enqueue one job per pointer sample.

# Snapshotting

Worker input is immutable snapshot/value data. Canonical document mutation stays serialized through transaction boundary.

# Python

Python coordinates and displays jobs. CPU parallelism occurs native. Asyncio may serve MCP/network/process coordination but not replace native scheduler.

# Shutdown

Stop accepting jobs -> cancel noncritical -> wait bounded -> flush save/recovery critical operations according policy -> destroy scheduler.

# Tests

Race/TSan, cancellation at every phase, queue saturation, priority inversion, stale publication, shutdown during export/save.