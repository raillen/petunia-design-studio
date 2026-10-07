# 18.15 — Stroke Width / Profile Tool Specification

# Purpose

Edit variable stroke width/profile on existing vector strokes.

# Representation

StrokeProfile is normalized along path 0..1 with profile control points and interpolation. It is appearance data separate from path nodes.

# Interaction

On-canvas handles at profile points; drag normal to path changes half/full width according model. Drag along path repositions profile point. Double-click adds; Delete removes non-required point.

# Context

base width, cap/join, profile preset, pressure/profile graph, scale-with-object.

# Constraints

Width nonnegative; cusp/path tangent degeneracies handled with stable normal fallback. Closed-path seam interpolation explicit.

# Commands

SetStrokeWidth, Add/Move/DeleteProfilePoint, ApplyStrokeProfilePreset, ExpandStroke.

# Accessibility/tests

Profile graph/table numeric alternative. Test closed path seam, zero width, sharp corners, dashed variable stroke and expand equivalence.