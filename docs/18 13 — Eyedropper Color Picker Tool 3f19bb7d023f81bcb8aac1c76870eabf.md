# 18.13 — Eyedropper / Color Picker Tool

# Identity

ToolId ptnd.tool.eyedropper.

# Modes

Document source, composited document, display sample, average radius; optional appearance pickup.

# Sampling

Pointer hover can preview value in Info/Color; click commits chosen fill/stroke color target only when pickup mode is active. Sampling itself is view-only.

# Source sample

If exact vector/object source beneath pointer is resolvable, report canonical ColorValue/SwatchRef. Composite mode samples rendered working-space image before display transform. Display mode explicitly labels monitor-converted value.

# Average

Radius defined in screen or document pixels by mode; averaging done in appropriate linear/working representation.

# Appearance pickup

Modifier can capture full fill/stroke/appearance into clipboard-like staged style, then apply through separate command/click if product enables.

# Context

sample mode, average radius, current/all layers for raster, target fill/stroke, appearance toggle.

# Commands

SetFillColor/SetStrokeColor/ApplyAppearance only on application; simple read has no history.

# Tests

CMYK/spot source, transparency, effects, raster profile, average radius, monitor proof mode and high DPI.