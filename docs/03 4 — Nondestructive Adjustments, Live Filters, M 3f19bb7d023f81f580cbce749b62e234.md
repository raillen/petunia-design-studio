# 03.4 — Nondestructive Adjustments, Live Filters, Masks, Blend & Compositing Workflow

# Principle

Professional Photo editing defaults to nondestructive operations where practical.

# Adjustment layers/nodes

Levels, Curves, Exposure, Brightness/Contrast, White Balance, HSL, Vibrance, Black & White, Channel Mixer, Gradient Map, Threshold/Posterize and Selective Color according color support.

# Live filters

Blur/sharpen/high-pass/noise/vignette/distortion and other supported effects are effect nodes with editable parameters.

# Scope

Adjustment/filter can apply as layer in stack, clipped to target or nested inside group/object according layer semantics.

# Masks

Each node can receive pixel/vector mask. Mask thumbnail focus makes edit target explicit.

# Blend

Opacity/blend mode on effect layer/node uses common compositor semantics.

# Preview

Parameter scrubbing stages one transaction and reuses caches/ROI. Expensive effect previews adapt quality under interaction then converge on idle.

# Destructive alternative

Apply Filter to Pixels / Merge / Rasterize are separate explicit actions. Dialog clearly labels editability loss.

# History

Adding effect one command; continuous parameter session coalesces; reorder effect/layer separately undoable.

# Export

Nondestructive nodes evaluate final quality. Unsupported external format receives degradation item.