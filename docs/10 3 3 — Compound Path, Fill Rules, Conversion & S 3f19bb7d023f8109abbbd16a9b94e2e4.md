# 10.3.3 — Compound Path, Fill Rules, Conversion & Style Inheritance

# Compound

CompoundPath groups contours/subpaths under one fill-rule/appearance without computing boolean union.

# Creation

Make Compound from selected compatible paths/shapes by evaluating parametric shapes or owning child subpaths according chosen canonical model. Release restores independent objects preserving world transforms/order where possible.

# Fill rules

EvenOdd/NonZero explicit. Reversing contour affects NonZero winding but not hidden inferred hole tags.

# Conversion

Parametric shape to curves creates VectorPath; compound release never rasterizes.

# Style

Creating compound uses key/front object appearance by deterministic policy. Original individual appearances cannot coexist in one simple compound unless appearance system represents per-subpath styles; otherwise operation reports loss/uses keep originals option.

# Export

SVG compound maps multi-subpath path/fill-rule exactly where supported.

# Tests

Nested holes, mixed winding, transforms, style differences, release roundtrip and boolean distinction.