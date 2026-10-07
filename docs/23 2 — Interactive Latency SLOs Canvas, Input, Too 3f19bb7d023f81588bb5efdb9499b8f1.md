# 23.2 — Interactive Latency SLOs: Canvas, Input, Tools, Snapping & Panels

# Tier R initial budgets

Frame interval normal canvas pan/zoom: p50 <=8 ms, p99 <=16.7 ms.

Pointer to transform preview: p50 <=8 ms, p99 <=16.7 ms.

Brush sample to visible dab: p50 <=12 ms, p99 <=24 ms.

Node/shape handle drag preview: p99 <=16.7 ms typical fixture.

Hit-test 100k simple objects after index warm: p95 <=4 ms.

Snap query representative indexed document: p95 <=4 ms.

Command palette query 10k actions/help/settings: p95 <=50 ms.

Layer tree continuous scroll 10k rows: target 60 Hz with no full-model reset.

Property multi-edit 1k objects: initial preview <=50 ms or async progress if operation exceeds budget.

# GUI thread

Task >8 ms recorded in dev diagnostics. >50 ms is a stall requiring justification/fix. No file decode/export/filter runs synchronously on GUI thread.

# Input backlog

Pen event queue latency monitored; coalescing may discard redundant hover/mouse move but not pressure/time samples necessary for brush semantics.

# Degraded mode

Under load, preview quality may drop but interaction must stay responsive; status can expose reduced preview only if noticeable/important.