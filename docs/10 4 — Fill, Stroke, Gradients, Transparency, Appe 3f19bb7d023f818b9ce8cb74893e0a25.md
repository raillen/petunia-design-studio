# 10.4 — Fill, Stroke, Gradients, Transparency, Appearance, Effects & Blend Modes

# Appearance Stack

Each drawable owns ordered appearance entries: Fill, Stroke and Effect groups plus object opacity/blend. Multiple fills/strokes allowed. EntryId is stable for selection/editing.

# Solid fill

ColorValue or SwatchReference. Setting global swatch reference remains linked until explicit detach.

# Gradients

Linear, radial, conical baseline; mesh gradient post-V1 or V1 by milestone. Gradient has stops with StopId, position, midpoint/interpolation metadata, semantic ColorValue and opacity.

# Gradient UI

On-canvas axis/ellipse handles; click line adds stop; drag stop moves; drag away deletes only with clear affordance; Color panel targets active stop. Context bar type/spread/reverse.

# Image/pattern fill

ResourceId + transform/tiling mode. On-canvas transform controls. Missing resource shows placeholder but retains transform/reference.

# Stroke

Width, alignment, cap, join, miter, dash array/offset, arrowheads, pressure/profile. Width may use document units; UI preview.

# Variable width

Profile graph normalized along path. Native evaluator interpolates width; on-canvas width points can be stored as profile control data if supported.

# Transparency

Object/appearance opacity plus gradient opacity masks. Transparency tool edits mask transform/stops, not fill.

# Blend modes

Normal, Multiply, Screen, Overlay, Darken/Lighten, Color Dodge/Burn, Hard/Soft Light, Difference, Exclusion, Hue/Saturation/Color/Luminosity baseline as color model permits. Blending definition and premultiplication tested against reference fixtures.

# Effects

Shadow, inner shadow, glow, blur, outline and future effects are typed EffectNodes. Non-destructive, reorderable where semantics permit. Expensive parameters update preview native, one commit transaction.

# Layer blend/options

Object/Group isolation/pass-through policy explicit. Knockout/advanced blend post-V1 unless specified.

# GUI

Appearance panel list; Color/Stroke panels edit selected entry. Small fill/stroke swatches in toolbar allow target switch, swap and none. Effects dialog/panel is nonmodal if live editing benefits.

# Commands

AddAppearanceEntry, Remove, Reorder, SetFill, SetGradientStop, SetStroke, SetBlendMode, SetOpacity, AddEffect, SetEffectParams, ToggleAppearanceEntry.

# Serialization

Every typed entry/version in document JSON; resource references by IDs; UI selected entry not persisted in document.

# Export

Adapter analyses each appearance entry; unsupported effect can preserve, expand, rasterize or reject via degradation plan.

# Tests

Gradient interpolation, stop order, alpha, stroke expansion equivalence, blend golden corpus, nested group compositing and export degradation mapping.