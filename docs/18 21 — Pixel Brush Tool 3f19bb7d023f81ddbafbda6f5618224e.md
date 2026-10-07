# 18.21 — Pixel Brush Tool

# Identity

ToolId ptnd.tool.pixel_brush. Shortcut B.

# Target

Explicit PixelTarget: RasterLayer pixels or compatible pixel mask/channel. Target shown in context/status; invalid target disables with reason.

# States

Idle -> Hover -> StrokeBegin -> StrokeActive -> Commit/Cancel. Pointer capture mandatory during stroke.

# Input

Pen/mouse batches include position/time/pressure/tilt/rotation. Python forwards batches; C++ BrushStrokeSession performs resampling/dabs/blend.

# Context

Brush preset, size, hardness, opacity, flow, spacing, blend mode, stabilizer, dynamics shortcut, target indicator.

# Cursor/HUD

Outer diameter, inner hardness indicator, orientation where relevant; modifier drag can adjust size/hardness with numeric HUD.

# Commit

One CommitBrushStroke command with staged tile set/delta. Esc/capture loss cancels before commit according state.

# Color

Paint ColorValue converted through target/document ColorEngine; no implicit sRGB assumption.

# Performance

Input backlog bounded; visible stroke latency within interactive budget.

# Tests

Pressure, tilt, high-DPI, tile boundaries, masks, high bit depth, profiles, long stroke, undo.