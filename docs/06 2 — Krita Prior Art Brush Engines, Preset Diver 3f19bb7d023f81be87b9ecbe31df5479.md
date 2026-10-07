# 06.2 — Krita Prior Art: Brush Engines, Preset Diversity & Raster Workflow Lessons

# Why study Krita

Krita demonstrates that a professional painting editor benefits from multiple brush-engine behaviors and rich per-engine parameters rather than one monolithic “brush” abstraction.

# Adopt

BrushPreset must declare capabilities/engine/operator rather than assuming every preset has same knobs.

Brush Settings UI should be schema-driven enough to show only parameters meaningful to selected brush engine.

Stabilization/smoothing is a tool/input concern separable from the artistic dab engine.

Clone, smudge/filter-like painting can reuse common input/brush infrastructure while retaining distinct operator semantics.

# Petunia baseline

V1 starts with a strong Pixel Brush engine and operator variants, not dozens of engines. Architecture keeps BrushEngine/BrushOperator descriptors so future engines can be added without rewriting Tool/Panel contracts.

# Avoid

Exposing hundreds of controls by default. Petunia uses common context controls + progressive advanced Brush Settings and searchable presets.

# Reference observations

Krita documentation exposes numerous brush engines including Pixel, Clone, Color Smudge, Filter, Particle, Spray and others, illustrating why capability-based preset UI matters.

# Sources

- [Krita Brush Engines documentation](https://docs.krita.org/en/reference_manual/brushes/brush_engines.html)
- [Krita Sketch Brush Engine](https://docs.krita.org/en/reference_manual/brushes/brush_engines/sketch_brush_engine.html)

# Petunia consequence

Brush preset schemas include engine/operator capability and the UI never assumes unsupported parameters.