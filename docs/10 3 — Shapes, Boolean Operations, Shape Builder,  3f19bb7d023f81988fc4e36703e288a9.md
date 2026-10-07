# 10.3 — Shapes, Boolean Operations, Shape Builder, Offset, Outline & Compound Paths

# Parametric shapes

Rectangle, rounded rectangle, ellipse, polygon, star, triangle, diamond, trapezoid, pie/donut, line/arrow, cog and future shapes implement ShapeKind + typed ShapeParameters. Canonical shape remains parametric until Convert to Curves.

# Creation

Tool drag maps pointer gesture to geometry. Shift/Alt modifiers affect constraint/center. Context fields update staged shape live. Pointer up commits CreateShapeCommand.

# Live handles

Rounded radius, star inner radius/points, pie angles, cog teeth etc expose semantic handles. Dragging modifies parameters through SetShapeParametersCommand, not raw path nodes.

# Convert to Curves

Evaluates current shape to VectorPath preserving appearance and transform as closely as possible. Destructive semantics explicit but undoable.

# Boolean baked

Add/Union, Subtract, Intersect, XOR, Divide operate on evaluated fill geometry. Source handling policy: replace inputs by result, optionally keep originals via explicit option. Style inheritance rule documented per operation.

# Live Boolean

Canonical LiveBooleanNode contains operation + operand references/children. Derived geometry evaluates through boolean engine. Layers panel presents result with nested operands; users can enter operand editing and Bake.

# Compound Path

Multiple contours/child paths share fill-rule semantics without boolean geometry merge. Release Compound restores children/contours according creation model.

# Shape Builder

Given eligible selected vector shapes, geometry engine planarizes/intersects and derives atomic regions. UI hover highlights region; drag paints region set. Add/Subtract mode accumulates staged region selection. Commit reconstructs output paths.

# Offset Path

Positive/negative offset, join, miter and cleanup. Live Offset effect stores parameters; baked command outputs path.

# Outline Stroke

Expand stroke into filled geometry using same stroke evaluator as renderer. Variable width/dashes/arrowheads result semantics explicitly tested.

# Divide

Splits overlapping shapes into non-overlapping pieces retaining z-order/style policy. Can create many objects; preflight warns huge result count.

# Fill rules

Nonzero/even-odd stored explicitly. Boolean/compound operations do not silently reinterpret.

# GUI

Boolean controls available contextual toolbar + Pathfinder panel. Icons show operations with tooltip/shortcut. Live toggle labelled. Shape Builder uses candidate highlight and HUD mode. Properties exposes exact operation/parameters.

# Commands

CreateShape, SetShapeParameters, ConvertShapeToPath, BooleanBake, CreateLiveBoolean, BakeLiveBoolean, CreateCompound, ReleaseCompound, ShapeBuilderCommit, CreateLiveOffset, BakeOffset, ExpandStroke.

# Performance

Boolean/shape builder native and cancellable for complex selection. Preview can use simplified/partial derivation; commit full quality.

# Tests

Golden geometry, fill rule, style inheritance, identical/coincident edges, holes, nested operands, live invalidation and renderer-vs-expanded-stroke equivalence.

[10.3.1 — Parametric Shape Schema Catalog & Evaluation Contract](10%203%201%20%E2%80%94%20Parametric%20Shape%20Schema%20Catalog%20&%20Evaluat%203f19bb7d023f81fb89b0f28036d72fbd.md)

[10.3.2 — Live Shape Handles, Constraint Solving & Parameter Editing](10%203%202%20%E2%80%94%20Live%20Shape%20Handles,%20Constraint%20Solving%20&%20%203f19bb7d023f81628aafe296c0d4246c.md)

[10.3.3 — Compound Path, Fill Rules, Conversion & Style Inheritance](10%203%203%20%E2%80%94%20Compound%20Path,%20Fill%20Rules,%20Conversion%20&%20S%203f19bb7d023f8109abbbd16a9b94e2e4.md)