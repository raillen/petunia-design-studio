# 09.7.3 — Brush Pipeline: Sampling, Resampling, Stabilizers, Dynamics, Dabs, Blending & Determinism

# Input sample

position, monotonic timestamp, pressure, tiltX/Y, rotation, tangential pressure, device/tool flags.

# Resampling

Convert irregular device events to stroke-distance/time samples. Spacing expressed relative to effective brush diameter or absolute policy. Algorithm must not change dramatically with event frequency.

# Stabilizers

Off; Simple smoothing; Weighted/velocity adaptive; Rope/delayed stabilizer. Each has explicit latency/path semantics and preview line descriptor.

# Dynamics

Curve mapping input axes pressure/tilt/velocity/direction/random → output size, opacity, flow, rotation, scatter, spacing, texture phase and operator-specific controls.

# Dab

Brush tip mask sampled/generated at position/scale/rotation. Scatter may generate multiple sub-dabs. Texture coordinates define canvas/stroke/tip anchoring mode.

# Opacity vs flow

Opacity caps stroke contribution; flow controls per-dab accumulation. Reference equations documented per blend/operator to avoid UI values becoming arbitrary.

# Randomness

PCG/xoshiro-class deterministic generator candidate; stroke seed captured at begin. Same preset+samples+seed must reproduce reference pixels within defined backend tolerance.

# Tile batching

Compute affected tile bbox per dab batch; group work by tile; avoid locking/unlocking per dab. Damage region aggregates for renderer.

# GPU/CPU

CPU reference path is semantic oracle. GPU path may batch compute/render but must meet error tolerance and brush ordering semantics.

# Commit

BrushStrokeSession owns staged tile results/COW deltas. pointer up finalizes one Command; cancel discards staged writes.

# Tests

Different event rates same trajectory, pressure curves, stabilizer behavior, scatter seed, tile seams, premultiplied alpha, high-bit-depth, long stroke backlog and GPU/CPU differential.