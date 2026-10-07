# 21.01 — Common Effect Node Contract, ROI, Quality, Color & Parameter Schema

# EffectDescriptor

EffectId/AdjustmentId; schemaVersion; category; supported pixel/color models; deterministic flag; CPU reference availability; GPU capability; ROI function; quality tiers; parameter schema; export capability.

# Evaluation

Input is immutable image/composite region plus color context, mask, opacity/blend and parameters. Output cannot mutate upstream tiles.

# Parameters

Typed PropertySchema with unit, range, default, clamp/reject behavior, animation/live-edit capability and serialization field name. Unknown fields handled by schema version migration.

# ROI

Each effect defines required source region for desired output ROI. Point effects ROI=output; neighborhood effects expand. Unbounded transforms declare mapping function.

# Edge modes

Clamp, Mirror, Wrap, Transparent and Constant only where meaningful. Default fixed per effect.

# Alpha

Effect declares whether it operates premultiplied color, unassociated color, alpha, luminance or geometry. Conversion around algorithm is explicit.

# Color

Effect declares working representation: scene/render linear RGB, perceptual Lab-like, native channel, or model-specific. Unsupported document mode is disabled or requires explicit conversion.

# Quality

InteractiveFast may approximate kernel/sample count/resolution; ExportFinal is normative. FinalView converges after idle.

# CPU/GPU

CPU reference defines semantics for golden/differential tests. GPU output must meet numeric/perceptual tolerance.

# Mask/blend

Effect result can be masked and composited with node opacity/blend through shared compositor, not custom ad-hoc alpha.

# Tests

Parameter boundaries, ROI, edges, alpha, color models, CPU/GPU, serialization/migration and preview-final convergence.