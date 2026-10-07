# 18.03 — Pen Tool Specification

# Identity

ToolId ptnd.tool.pen. Persona Design. Shortcut P.

# State machine

Idle -> BuildingContour -> DraggingNewHandle -> AdjustingHandle -> ClosingCandidate -> Finished/Cancelled.

# Input semantics

Click creates a corner node. Drag creates a smooth node with opposed handles. Clicking the first node closes. Clicking a compatible endpoint can continue an existing path after explicit target resolution.

# Modes

Pen, Polygon and optional Smart. Polygon commits lines. Smart may infer handles but stores ordinary path semantics.

# Modifiers

Shift constrains handle/segment angle. Alt/Option breaks tangent relation. Temporary Node modifier adjusts a previous handle without leaving Pen.

# Preview

Rubber-band segment, close-node highlight, snap target, tangent handles and length/angle HUD.

# Context

Mode, fill/stroke summary, snapping, new-node type and continue-path behavior.

# Commit/cancel

A logical path creation is one history transaction where practical. Esc cancels the latest uncommitted stage before abandoning the whole path.

# Commands

CreatePath, AppendPathSegment, FinishPath or equivalent staged transaction builder.

# Tests

Open/closed paths, endpoint continuation, modifier timing, snapping, capture loss, empty path cleanup and undo grouping.