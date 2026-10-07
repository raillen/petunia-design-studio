# 18.32 — Dodge / Burn Tools

# Identity

Separate ToolIds or one grouped tool with operator Dodge|Burn.

# Purpose

Locally lighten/darken tonal ranges.

# Parameters

range Shadows|Midtones|Highlights; exposure/strength; protect tones/color; brush size/hardness/flow; nondestructive/direct target mode.

# Math

Exact tonal weighting function defined in effect/operator schema, applied in chosen working luminance model. Avoid naïve RGB add/subtract that shifts hue unexpectedly unless compatibility mode requires.

# Nondestructive mode

Preferred option can paint into dedicated DodgeBurnLayer/mask carrying operator strokes. Direct pixel mode clearly labeled destructive.

# Context

operator, range, exposure, protect tones, target, brush common controls.

# Commit

Brush stroke transaction; live-layer creation separate command.

# Tests

Neutral ramps, saturated colors, masks, repeated strokes, high bit depth, color profile and undo.