# 18.28 — Refine Selection Workspace Specification

# Entry

Action ptnd.selection.refine requires nonempty PixelSelection and raster/composite source.

# Dedicated state

RefineSession snapshots selection and source revision. Workspace/panel shows preview modes: marching ants, overlay, black/white, on layers.

# Parameters

radius, feather, smooth, contrast and any decontaminate/edge-color feature only if exact algorithm specified. Refine Brush marks areas needing edge recalculation/foreground-background hints.

# Output

New Selection, Pixel Mask on target, New Layer with Mask or other supported explicit target. Output choice visible before Apply.

# Preview

Lower-resolution adaptive analysis permitted; final Apply computes full quality. Cancel restores exact input selection.

# Stale source

Source change invalidates refine result; do not apply against wrong revision silently.

# Accessibility

Parameter UI fully keyboard accessible; refine brush itself pointer-optimized. Preview modes named/announced.

# Tests

Hair/fur, hard object edge, transparent subject, cancel, stale revision and output modes.