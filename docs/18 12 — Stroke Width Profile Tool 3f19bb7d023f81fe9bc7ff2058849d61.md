# 18.12 — Stroke Width / Profile Tool

# Identity

ToolId ptnd.tool.stroke_profile.

# Purpose

Edit stroke width and variable-width profile directly on canvas.

# Target

Selected path/stroke Appearance entry. Multiple compatible strokes can show shared controls; mixed profiles handled explicitly.

# Interaction

Width handles attach to centerline positions. Drag perpendicular to tangent changes local width multiplier; position can move along path. Profile curve shown in Stroke panel.

# Profile

Control point stores normalized arc-length position, width multiplier and interpolation metadata. End behavior defined.

# Context

Base width, selected profile point position/value, cap/join, reset profile, smooth/corner profile interpolation.

# Commands

SetStrokeWidth, AddProfilePoint, MoveProfilePoint, SetProfilePoint, RemoveProfilePoint, ResetStrokeProfile.

# Geometry

Arc-length mapping and outline evaluation in C++ StrokeEngine. Handles remain screen-readable at zoom extremes.

# Accessibility

Profile point table/graph keyboard editing and numeric values.

# Tests

Closed path seam, transformed path, nonuniform scale, profile crossing, expansion equivalence and undo.