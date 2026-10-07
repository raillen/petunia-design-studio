# 18.02 — Node Tool Specification

# Identity

ToolId ptnd.tool.node; default shortcut A.

# Targets and states

VectorPath or path-editable object. Idle → PathHover → Node/SegmentHover → NodeSelection → DragNodes | DragHandle | MarqueeNodes | InsertNode → Commit/Cancel.

# Selection

Click node selects; Shift toggles; marquee selects subnodes. NodeIds live in view subselection and remap via TopologyChangeMap.

# Node kinds

Cusp, Smooth, Symmetric. Smooth keeps tangent collinear; Symmetric mirrors direction and length.

# Handles

Drag control handle updates cubic preview; Shift angle-snap; Alt/Option temporarily breaks linkage if convention enabled. Anchor drag moves associated handles with node.

# Segment editing

Double-click segment inserts exact De Casteljau node at nearest t. Delete and Delete Preserving Curve are distinct actions.

# Context

node type, join, break, close/open, reverse, segment line/curve, delete-preserve, snapping.

# Commands

EditPathNodes, SetNodeType, InsertNode, DeleteNodes, JoinEndpoints, BreakPath, CloseContour, OpenContour, ReverseContour.

# Accessibility/tests

Keyboard cycle/nudge nodes; numeric coordinates/handles in Properties. Test open/closed contours, invariants, insertion exactness, reverse, multiple transformed paths, ID preservation and undo.