# 18.26 — Lasso / Polygonal / Magnetic Selection Tools Specification

# Freehand lasso

Pointer trace closes on release; native rasterization creates soft/binary selection boundary according anti-alias/feather.

# Polygonal

Click vertices, Backspace removes last, Enter/double click closes, Esc cancels current polygon.

# Magnetic

Edge-analysis service proposes anchors/segments near pointer. User clicks to pin anchors. UI shows confidence/path preview; Backspace removes anchor.

# Combine

New/Add/Subtract/Intersect same as all selection tools.

# Performance

Magnetic analysis coalesces pointer updates, may operate on local image pyramid and never blocks GUI.

# Staleness

If sampled composite revision changes during long magnetic session, either keep captured snapshot until commit or invalidate with explicit rule.

# Tests

High/low contrast edges, complex hair-like edge, polygon cancellation, combine modes, selection mask seams.