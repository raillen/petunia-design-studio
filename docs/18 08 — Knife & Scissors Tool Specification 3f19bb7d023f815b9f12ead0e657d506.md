# 18.08 — Knife & Scissors Tool Specification

# Identity

ToolIds ptnd.tool.knife and ptnd.tool.scissors. Persona Design.

# Knife

Captures freehand or straight cut path. Native geometry computes intersections and topology reconstruction.

# Scissors

Click nearest eligible segment/node to split at precise parameter.

# State machine

Idle -> HoverPath -> CaptureKnifePath/ScissorArmed -> IntersectionPreview -> Commit/Cancel.

# Context

Knife Straight/Freehand; scope Selection/Topmost/AllEligible; close-result policy; keep originals where meaningful. Scissors exposes Split/Open/Separate.

# Overlay

Cut path, intersection markers, prospective pieces and invalid/coincident warnings.

# Commands

KnifeCut, SplitPathAt, OpenContour, SeparateContours.

# IDs

Surviving nodes retain NodeIds where possible; new intersections receive new IDs.

# Tests

Tangential/coincident/self-intersecting geometry, multi-contour paths, many intersections, cancel and undo.