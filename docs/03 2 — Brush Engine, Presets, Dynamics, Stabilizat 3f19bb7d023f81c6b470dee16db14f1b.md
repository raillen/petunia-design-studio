# 03.2 — Brush Engine, Presets, Dynamics, Stabilization, Texture & Painting Semantics

# Brush definition

BrushPreset references:

- tip shape/image;
- spacing;
- hardness/falloff;
- scatter;
- rotation;
- texture;
- size/opacity/flow;
- dynamics curves;
- blend/operator;
- smoothing/stabilizer;
- deterministic random policy.

# Input

Samples preserve timestamp, pressure, tilt, rotation and device. Native resampling creates consistent stroke spacing across event-rate differences.

# Stabilization

Modes may include simple smoothing, weighted moving path and delayed rope/stabilizer. UI previews lag/rope clearly so perceived latency is intentional.

# Dynamics

Map pressure/tilt/velocity/random to size, opacity, flow, rotation/scatter and other supported axes. Curves editable in Brush Settings.

# Dabs

Native C++ engine generates masks/texture and blends only affected tile regions. Python never receives per-dab callbacks.

# Preview

Cursor ring shows effective size/hardness/angle. HUD numeric change via modifier/drag may update settings without opening panel.

# Preset edits

Changing preset settings creates temporary modified state; Save New/Update preset explicit. Document stores brush operation result, not dependency on current future preset unless live paint model explicitly introduced.

# Reproducibility

Stroke records sufficient seed/settings snapshot for undo/replay/evidence when implementation uses procedural randomness.

# Performance

Sample ingestion bounded, brush queue backlog measured, tile batching and SIMD/GPU paths benchmarked.