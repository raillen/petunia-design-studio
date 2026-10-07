# 29.2 — VectorPath, ParametricShape, LiveBoolean & Appearance Schemas

# VectorPathRecord

Common Object + contours[], fillRule, appearance. Contour has ContourId, closed, ordered PathNode. PathNode has NodeId, position, in/out handles, NodeKind.

# ParametricShapeRecord

Common Object + ShapeKindId + schemaVersion + typed parameters + appearance. Evaluated path is derived. ConvertToCurves produces VectorPath.

# LiveBooleanRecord

Operation, ordered operand child/ref IDs, evaluation options and result appearance policy. Operand geometry remains canonical; result derived until Bake.

# AppearanceStack

Ordered AppearanceEntry with EntryId and tagged union Fill, Stroke, Effect. Object opacity/blend outside entries unless effect semantics require otherwise.

# Fill

Solid ColorValue/SwatchRef; GradientDefinition; Image/Pattern resource fill and transform.

# Stroke

Width, alignment, caps/joins/miter, dash, offset, variable profile, markers/brush style refs.

# Effect

EffectId/schemaVersion + typed params + enabled/mask/ref data where relevant.

# Validation

Contour/node IDs unique within object/document policy; finite geometry; valid gradients/stops; no missing resources; live boolean cycles forbidden.