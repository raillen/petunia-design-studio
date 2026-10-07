# 18.36 — Raster Gradient Tool

# Modes

Destructive pixel gradient and nondestructive generator/fill-layer mode. UI makes mode explicit.

# Interaction

Same geometric handle grammar as vector gradient: start/end/focus/radius and stops.

# Pixel mode

At commit, raster engine evaluates gradient only in affected bounds/selection and composites into PixelTarget with chosen blend/opacity.

# Live mode

Creates Generator/Fill layer/node with GradientDefinition; geometry remains editable.

# Context

mode live/pixels, gradient type, spread, stops, opacity/blend, target.

# Commands

ApplyRasterGradient or CreateGradientLayer/SetGradientParameters.

# Tests

Selection edges, high bit depth, transformed layer, live edit, alpha, color-managed stops and undo.