# 18.23 — Pixel Brush Tool Specification

# Identity

ToolId ptnd.tool.pixel_brush; default shortcut B.

# Target

Explicit PixelTarget required: RasterLayer pixels or editable PixelMask. If current selection is vector/text, tool disables with reason and offers explicit raster/mask workflow.

# State machine

Idle → Hover → StrokeArmed → Painting → FinalizingTiles → Commit/Cancel.

# Input

Pointer/pen samples batch to native BrushStrokeSession with position, pressure, tilt, rotation, time and device metadata. Main thread never processes dabs.

# Cursor

Screen-space outline represents effective brush diameter, hardness/angle where practical. While changing size/hardness by modifier drag, HUD shows numeric values.

# Context

preset, size, hardness, opacity, flow, spacing, smoothing/stabilizer, blend/operator, pressure toggles and target indicator.

# Selection

Active PixelSelection limits write coverage. Selection snapshot captured at stroke begin so selection cannot shift mid-stroke unexpectedly.

# Brush session

Preset snapshot + deterministic seed + target revision. Native engine resamples/dynamics/dabs and tracks dirty tiles/damage.

# Commit

Pointer up finalizes one CommitBrushStroke transaction with TileCOWDelta. Cancel/lost-capture discards staged tile versions.

# Straight line

Shift-click/defined gesture can draw line from last committed point only if visible status/preset semantics are consistent.

# Accessibility

Brush is pointer/pen-first; every parameter is keyboard editable. Non-pointer equivalent is not required for freehand painting, but target, preset and size are accessible.

# Tests

Pressure/tilt, selection edge, mask target, tile seams, lost capture, 10-minute stroke, deterministic seed, undo/redo and 8/16/float pixels.