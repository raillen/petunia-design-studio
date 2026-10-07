# 25.2 — Provisional Interactive Latency and Frame-Time Budgets

# Recommended-tier provisional SLOs

These are initial engineering targets and become evidence-calibrated after first vertical slice.

- idle canvas pan/zoom frame p50 ≤ 8.3 ms; p99 ≤ 16.6 ms for standard fixture.
- pointer input → visible transform preview p50 ≤ 8 ms; p99 ≤ 20 ms.
- brush sample ingestion backlog ≤ 1 frame under standard brush; visible stroke p99 ≤ 20 ms.
- hit test among 100k indexed objects p95 ≤ 4 ms.
- snapping query standard dense fixture p95 ≤ 4 ms.
- QAction/tool activation UI response ≤ 50 ms perceived, no blocking task.
- context panel selection update ≤ 50 ms for ordinary selection.
- Layers scroll 10k rows sustains display refresh without >50 ms main-thread stalls.

# Low-tier

May target 30/60 fps profiles based fixture, but pointer feedback p99 should remain <33 ms for core interactions.

# Main-thread stall

No routine background operation may block GUI thread >8 ms; >16 ms events are traced and treated regression candidates.

# Quality adaptation

Interactive quality may reduce but must converge within ~150 ms idle trigger or effect-specific budget.

# Measurement

Instrument timestamp at normalized input, native processing, frame submit and present where platform permits.