# 09.7.3 — Brush Stroke Pipeline, Sample Resampling, Dynamics, Dabs & Determinism

# Input sample

```
BrushInputSample {
  position_document
  timestamp
  pressure
  tilt_x/tilt_y
  rotation
  tangential_pressure
  device_id
}
```

# Stroke session

begin(target,preset_snapshot,seed,selection_snapshot,transform)

add_samples(batch)

preview_state()

commit()/cancel()

# Resampling

Raw event rate varies by tablet/OS. Engine reconstructs a stable path using timestamps and distance; dab spacing is based on brush-space distance, not event count.

# Stabilizers

Each stabilizer is strategy with state:

- None;
- smoothing filter;
- weighted smoothing;
- rope/delay.

Preset stores parameters. Delayed stabilizer UI shows predictive/rope overlay from engine output.

# Dynamics

Property = base × curve(input) × modifiers. Inputs: pressure, speed, direction, tilt magnitude/azimuth, rotation, random, stroke progress. Curves are normalized piecewise/spline structures with deterministic evaluation.

# Dab generation

For each spacing position derive size, opacity, flow, angle, scatter offset, texture transform and operator. Random uses explicit PRNG algorithm/version + stroke seed for reproducibility.

# Blend

Brush operator receives destination span/tile region + dab mask/color and works in compositor-defined premultiplied/linear policy. Erase is explicit alpha/composite operator.

# Damage

Aggregate touched pixel bounds and TileCoords; render invalidation uses damage, not full layer.

# Commit

One transaction records tile modifications and brush metadata required for history/evidence, not thousands of per-dab Commands.

# Tests

Identical raw input => deterministic pixels on same engine version/backend reference; different event sampling of same path within tolerance; pressure curve fixtures; tile seam absence; cancel leaves no mutation; 10-minute stroke memory stability.