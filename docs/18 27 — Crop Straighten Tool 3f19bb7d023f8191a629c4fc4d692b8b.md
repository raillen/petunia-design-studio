# 18.27 — Crop / Straighten Tool

# Identity

ToolId ptnd.tool.crop.

# Default semantic

Nondestructive crop modifies Surface/view crop bounds or stores crop transform when possible. Destructive Trim is separate command.

# States

Idle/ExistingCrop -> AdjustRect -> Rotate/Straighten -> Commit/Cancel.

# Overlay

Outside dim, crop handles, center, rule-of-thirds/golden/grid options and horizon line.

# Context

aspect preset/free/custom, W/H, rotation/straighten angle, overlay grid, delete cropped pixels toggle only when explicitly destructive, reset.

# Straighten

Drag horizon line or numeric angle; resulting transform + crop computed without repeated resampling.

# Commit

SetCrop/SetSurfaceBounds nondestructive. TrimToCrop or ApplyRasterCrop stages new pixels and requires destructive wording.

# Constraints

Crop can extend beyond source for transparent/background area according mode. Locked Surface/document rules explicit.

# Tests

Rotated image, aspect presets, out-of-bounds, nondestructive reopen, destructive trim, export and undo.