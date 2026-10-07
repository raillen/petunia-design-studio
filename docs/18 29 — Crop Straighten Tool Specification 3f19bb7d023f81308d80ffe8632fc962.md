# 18.29 — Crop / Straighten Tool Specification

# Identity

ToolId ptnd.tool.crop.

# Modes

Non-destructive Surface/document crop default; destructive Trim Pixels separate action.

# Interaction

Crop rectangle handles, move inside, rotate/straighten control. Aspect presets Free/Original/1:1/custom. Grid overlays rule-of-thirds/diagonal/golden optional.

# Straighten

Drag horizon line or numeric angle. Operation composes view/object transform/crop semantics; avoids resampling canonical pixels until destructive apply or export.

# Context

aspect, width/height, units, rotation/straighten angle, overlay, delete cropped pixels toggle only if explicitly destructive and clearly labeled.

# Commit

SetCrop / SetSurfaceBounds and transform command. TrimToCrop performs separate raster tile operation with confirmation/preflight if data loss.

# Reset/cancel

Reset returns crop proposal to document bounds; Esc cancels staged crop.

# Tests

rotated image, transparent edges, non-destructive repeated recrop, destructive trim undo, aspect locking and export.