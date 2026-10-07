# 18.23 — Crop & Straighten Tool Specification

# Identity

ToolId ptnd.tool.crop. Photo/shared image workflow.

# Modes

Nondestructive Crop adjusts Surface/document crop semantics. Trim Pixels is a separate destructive action. Straighten is an angle/transform operation integrated with crop preview.

# State machine

Idle -> CropRectEdit -> Rotate/Straighten -> Preview -> Commit/Cancel.

# Interaction

Drag handles resize crop bounds. Drag interior moves bounds. Straighten can be drawn as horizon/reference line. Grid overlay selectable.

# Context

Aspect preset/custom, X/Y/W/H, rotation, overlay grid, lock aspect, delete-cropped-pixels destructive toggle only when explicitly selected.

# Canonical

Nondestructive crop stores bounds/transform metadata and leaves underlying raster untouched. Export evaluates visible region.

# Commands

SetCrop, SetSurfaceBounds, StraightenView/Content, TrimToCrop.

# Resampling

No pixel resample during nondestructive crop/straighten unless explicit apply/flatten requires it.

# Tests

Rotated image, crop outside content, transparency, repeated edits, trim vs nondestructive behavior, undo and save/reopen.