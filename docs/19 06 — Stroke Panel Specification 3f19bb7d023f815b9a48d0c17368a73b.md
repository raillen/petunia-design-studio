# 19.06 — Stroke Panel Specification

# Identity

PanelId ptnd.panel.stroke.

# Controls

width, alignment, cap, join, miter, dash sequence/offset, arrowheads, scale-with-object and variable profile.

# Target

Active StrokeEntry from Appearance. Multiple strokes require explicit target. Mixed selection values supported.

# Dash editor

Pattern list numeric fields with validation, preset list and visual preview. Odd dash count normalization follows vector standard policy.

# Profile

Mini graph/editor with control points plus numeric table. On-canvas Stroke Width tool synchronizes same StrokeProfile.

# Arrowheads

Start/end independently; scale/offset policy explicit. Expansion/export analyzer aware.

# Accessibility/tests

Cap/join buttons named, graph keyboard alternative. Test dashed variable stroke, inside/outside alignment and multi-appearance target.