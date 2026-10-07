# 18.12 — Eyedropper / Color Picker Tool Specification

# Identity

ToolId ptnd.tool.eyedropper. Shared Design/Photo.

# Modes

InspectOnly, ApplyColor, CopyAppearance. Sampling source can be SourceObject, CompositedDocument or DisplayAppearance; point or averaged radius.

# State machine

Idle -> HoverSample -> PressSample -> Apply/Inspect -> optional ReturnPreviousTool.

# Color semantics

Source sampling preserves semantic ColorValue where possible. Composite sampling occurs in declared render working space then converts to target. Display sampling is clearly labeled and never masquerades as source CMYK/spot.

# Context

Source, radius, target Fill/Stroke/GradientStop/Foreground, include transparency and copy-appearance mode.

# HUD/Info

Components, alpha, profile/color space and sampling mode.

# Commands

Inspect does not mutate. Apply invokes standard property/appearance command path.

# Tests

RGB/CMYK/Lab/spot, transparency, proof mode, averaging edges, transformed content and temporary-tool restore.