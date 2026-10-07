# 18.02 — Node Tool Specification

# Identity

ToolId: ptnd.tool.node. Persona: Design. Default shortcut: A.

# Targets

Editable VectorPath and explicitly compatible live-shape proxies.

# State machine

Idle -> PathHover -> NodeSelection -> NodeDrag | HandleDrag | MarqueeNodes | SegmentEdit -> Commit/Cancel.

# Hit priority

Selected node > handle > segment > unselected node/path using screen-space tolerance and hysteresis.

# Interaction

Click node selects; Shift toggles; marquee selects; node drag moves selected nodes; handle drag edits tangent; double-click segment inserts node at nearest parameter.

# Node kinds

Cusp, Smooth and Symmetric. Conversion preserves geometry whenever mathematically possible.

# Context

Node type, join, break, close/open, reverse, delete/delete-preserve, snapping and numeric coordinates.

# Commands

EditPathNodes, InsertNode, DeleteNodes, SetNodeType, JoinEndpoints, BreakPath, ClosePath and ReversePath.

# Selection

Sub-selection is view/session state keyed by NodeId and remapped after topology edits when nodes survive.

# Tests

Handle constraints, exact insertion, delete-preserve, multi-node transforms, NodeId stability, join/break and undo/redo.