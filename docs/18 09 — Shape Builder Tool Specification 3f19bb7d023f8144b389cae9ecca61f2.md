# 18.09 — Shape Builder Tool Specification

# Identity

ToolId ptnd.tool.shape_builder. Design.

# Preconditions

Two or more selected vector-capable operands. Raster/live text require explicit conversion first.

# Preparation

Native engine evaluates operands, computes planar arrangement and yields atomic RegionIds bound to operand revision fingerprint.

# State machine

Prepare -> Ready -> HoverRegion -> PaintAdd/PaintSubtract -> PreviewResult -> Commit/Cancel.

# Interaction

Hover highlights one region. Click toggles current mode. Drag paints regions once per gesture. Modifier temporarily flips Add/Subtract.

# Options

Keep originals, merge adjacent same-style regions, result grouping and style-source priority.

# Stale revisions

Any operand geometry change invalidates prepared regions and blocks stale commit.

# Commands

ShapeBuilderCommit with operand IDs/revisions, selected regions, mode and style policy.

# Tests

Holes, coincident edges, touching corners, many overlaps, stale edits, cancel and deterministic result.