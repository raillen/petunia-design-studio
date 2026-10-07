# 18.19 — Pixel Brush Tool Specification

# Identity

ToolId ptnd.tool.pixel_brush. Persona Photo. Default shortcut B.

# Target

Writable RasterLayer pixels or PixelMask selected explicitly. Vector/text target disables brush with reason and explicit rasterize/mask alternatives.

# State machine

Idle -> Hover -> StrokeCapture -> NativeFlushPending -> Commit/Cancel.

# Input

BrushStrokeSession receives sample batches containing document position, timestamp, pressure, tilt, rotation, device/buttons. Python never executes dab loop.

# Context

Preset, size, hardness, opacity, flow, spacing, blend/operator, smoothing/stabilizer, dynamics and current PixelTarget badge.

# Cursor

Screen-space outline represents effective size/hardness and rotation where meaningful. Cursor color/contrast adapts to artwork.

# Native pipeline

Resample -> stabilize -> dynamics -> dab generation -> texture/scatter -> blend -> dirty tile set -> render damage.

# Commit

Touched tiles are staged then published atomically as one history transaction. Cancel discards staged writes.

# Performance

Tier R visible-dab p99 <=24 ms initial SLO. Sample queue is bounded; required pen data cannot be dropped indiscriminately.

# Tests

Pressure/tilt, deterministic seed, tile boundaries, masks, high bit depth, color management, capture loss, memory budget and undo.