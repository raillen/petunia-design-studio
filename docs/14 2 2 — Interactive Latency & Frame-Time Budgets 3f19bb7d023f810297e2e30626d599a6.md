# 14.2.2 — Interactive Latency & Frame-Time Budgets

# Initial V1 engineering targets

These are budgets, not measured claims.

# Canvas

Pan/zoom clean fixture: p50 <= 8 ms frame CPU+GPU submit path; p95 <= 16.6 ms; p99 <= 33 ms on Tier R.

Transform preview 10k visible vector objects: pointer-to-visible update p95 <= 16.6 ms.

Snap/hit query typical scene: p95 <= 4 ms native query.

Selection marquee 100k indexed objects: query/update p95 <= 12 ms.

# Brush

Input event arrival -> visible stroke update p50 <= 8 ms, p95 <= 16 ms, p99 <= 25 ms on Tier R for standard 256px brush.

No sustained input backlog > 1 display frame under baseline brush.

Large 2k brush may degrade but UI remains responsive/cancellable.

# Text

Caret key/typing echo p95 <= 16 ms in typical story.

Reflow 10-page brochure edit near paragraph p95 <= 30 ms asynchronous convergence where necessary.

Font picker search first results <= 100 ms warm index.

# UI chrome

Panel open/switch <= 100 ms perceived for normal panel.

Layers scroll maintains display refresh target with 10k rows.

Command palette first filtered results <= 50 ms for built-in registry.

# Degradation

When budget exceeded due operation complexity, maintain pointer/UI responsiveness and move final work to cancellable job rather than blocking main thread.

# Measurement

Use input timestamp markers, Qt event timing, native spans and GPU timestamps; do not estimate by eye.