# 18.28 — Clone Tool

# Identity

ToolId ptnd.tool.clone.

# Source

Alt/Option-click sets source anchor in document coordinates on chosen sampling source. UI shows source crosshair and destination cursor.

# Modes

Aligned: source offset persists across strokes relative to destination.

Non-aligned: every new stroke begins from fixed source anchor.

Sample: Current Layer, Current & Below, All Layers.

# Brush

Uses BrushEngine mask/dynamics but pixel content comes from sampled immutable source snapshot.

# Transform

Sampling uses source-layer/document transforms consistently. Source snapshot revision fixed at stroke begin to avoid feedback unless “sample current stroke” is explicitly designed.

# Context

brush preset/size/hardness/opacity/flow; aligned; source scope; target indicator.

# Commit

One raster delta transaction per stroke.

# Errors

No source anchor -> cursor/status instructs set source; invalid/missing source cancels safely.

# Tests

Aligned/nonaligned, transformed source, sampling merged, masks, edge tiles, source=target overlap, undo.