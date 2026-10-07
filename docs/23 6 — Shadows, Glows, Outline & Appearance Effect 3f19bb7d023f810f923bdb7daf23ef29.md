# 23.6 — Shadows, Glows, Outline & Appearance Effects

# Drop Shadow

offset x/y or distance/angle, blur sigma/radius, spread/choke, ColorValue, opacity, blend mode. Shadow derived from source alpha/mask then spread/blur/color/composite.

# Inner Shadow

Same parameters but clipped/inverted to interior; formula and edge behavior CPU reference.

# Outer/Inner Glow

Color/gradient optional, radius, spread, opacity, technique. If gradient glow deferred, schema version reflects capability.

# Outline

Width, alignment, fill/ColorValue, join/corner semantics. Prefer reuse StrokeEvaluator where result equivalent.

# Ordering

Effects are ordered relative to appearance stack according explicit evaluation model. A drop shadow cannot randomly move before/after object fills across backends.

# Bounds

Each effect reports visual bounds/ROI expansion based on offset/radius/spread.

# UI

Effect panel groups common params, on/off, reset, duplicate, reorder where semantic. Canvas preview immediate.

# Tests

Alpha shapes, transparent holes, nested effects, clipped groups, extreme blur/spread and bounds correctness.