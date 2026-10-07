# 21.3 — Shadow, Glow, Outline & Appearance Effects

# Drop Shadow

Offset X/Y or distance+angle, blur radius, spread, color, opacity, blend mode, knock-out/source interaction and clipping policy.

# Inner Shadow

Same plus inside coverage semantics. Mask derives from source alpha/coverage; effect must behave consistently on vector/raster/text.

# Outer/Inner Glow

Radius, spread/choke, color/gradient if supported, opacity, blend. Inner glow coverage and edge-distance function documented.

# Outline

Width, position Inside/Center/Outside, join/corner policy, color/gradient and blend. For vector objects, appearance outline can use geometry; raster coverage requires morphological/distance evaluation.

# Effect order

Effects are ordered Appearance entries/effect chain. Reordering can change output and is canonical.

# Bounds/ROI

Every effect declares geometric bounds inflation and render ROI. Shadows must participate in object bounds/export correctly.

# Serialization

EffectId + schemaVersion + parameters + enabled state; backend shader layout derived.

# Tests

Transparent source, nested groups, masks, clipped effects, extreme spread/radius, vector/raster equivalence and export degradation.