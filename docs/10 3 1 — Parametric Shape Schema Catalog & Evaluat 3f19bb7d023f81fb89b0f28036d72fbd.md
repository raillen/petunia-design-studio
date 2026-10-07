# 10.3.1 — Parametric Shape Schema Catalog & Evaluation Contract

# Shape contract

Every ParametricShape stores ShapeKind, shapeVersion, dimensions/local transform, typed parameter set and appearance. Evaluated geometry is derived and deterministic.

# Baseline kinds

Rectangle, RoundedRectangle, Ellipse, Arc/Pie/Donut, Triangle, Diamond, Polygon, Star, Line, Arrow, Cog.

# Common rules

Dimensions nonnegative canonical; reverse pointer drag encoded via transform/origin rather than negative semantic width where possible. Parameter validation returns clamped preview only when UI communicates it; canonical commit stores valid values.

# Rectangle

width/height + four corner radii + corner style/link mode.

# Ellipse

width/height; partial modes add start/end angles and closure; donut adds innerRatio.

# Polygon/star

integer point count, outer radius/dimensions, rotation; star innerRatio; optional rounding parameters versioned.

# Arrow/line

line endpoints or local length/angle + start/end arrow shape refs, shaft width where shape—not stroke—semantics intended. Avoid duplicating Stroke arrowheads unless shape tool has distinct geometry.

# Cog

bounded tooth count and explicit root/tip proportions.

# Evaluation

ShapeEvaluator returns PathGeometry + semantic handle descriptors + bounds. Renderer/boolean/export use same evaluator.

# Tests

For each kind minimal/default/extreme params, save/load, Convert to Curves visual match and SVG/PDF mapping.