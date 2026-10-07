# 10.2 — Pen, Pencil, Nodes, Path Editing, Corners, Knife & Scissors

# Path data

VectorPath -> contours -> PathNode/segments. Node stores position plus in/out handles or equivalent segment representation. NodeId stable within object where topology operation can preserve it.

# Pen

State machine: Idle -> BuildingContour -> DraggingHandle -> AdjustingPreviousHandle -> Closing -> Finished/Cancelled.

Click adds corner; drag adds smooth; modifier breaks/adjusts handle; clicking start closes. Preview segment is derived. Enter finishes open contour, Esc cancels only uncommitted state.

# Smart/Polygon modes

Polygon forces line segments. Smart mode may infer smoothness but stores normal path semantics, not opaque algorithm state unless live path mode is deliberate.

# Pencil

Input batch -> smoothing/stabilizer -> curve fitting -> simplify within error tolerance. Live preview can be polyline; commit fits cubic path. Option "Sculpt existing stroke" identifies compatible path and replaces affected span transactionally.

# Node selection

Click, shift, marquee nodes. SubSelection belongs view. Selection survives geometry mutation via NodeId mapping where possible; deleted nodes removed cleanly.

# Node transforms

Move selected nodes, handles independently or linked by node type. Shift angle constraints. Snapping uses node/tangent/geometry targets.

# Node types

Cusp: handles independent.

Smooth: collinear directions, lengths independent.

Symmetric: collinear, equal lengths.

Auto/smart only if deterministic semantic mode is added.

# Insert/delete node

Double-click segment inserts at nearest parameter preserving curve shape mathematically. Delete default can alter path; "Delete preserving curve" fits/merges neighboring segments within tolerance.

# Join

Requires compatible open endpoints. UI previews pair when ambiguous. Join command may reverse contour direction automatically only if reported/consistent.

# Break

Selected node duplicates topology endpoint to split contour. Break at parameter inserts then splits if not existing node.

# Close/Open/Reverse

Close adds segment; Open removes chosen closing edge; Reverse flips contour orientation and swaps handles preserving geometry.

# Corner Tool

CornerModifier associated with eligible nodes and radius/type. Drag handle sets radius bounded by geometry; conflicting radii are resolved predictably. Bake converts to explicit curves.

# Knife

Gesture path transformed to document space; geometry engine computes intersections and splits affected paths/shapes. Options cut through all selected vs topmost, close resulting shapes, keep/remove segment. Preview markers and affected outlines.

# Scissors

Click exact path location -> insert split -> open contour / separate depending context. Snaps to existing nodes.

# Path offset/outline

Offset Path and Expand Stroke live or baked variants use geometry engine. UI exposes joins, miter, offset side, cleanup.

# Commands

CreatePath, AppendPathSegment, FinishPath, EditPathNodes, InsertNode, DeleteNodes, JoinEndpoints, BreakPath, ClosePath, ReversePath, SetNodeType, SetCornerModifier, BakeCorners, KnifeCut.

# Boundary

Python ToolController handles state/intention and HUD; C++ owns nearest-path query, curve math, fitting, intersections, topology and mutation command execution.

# Tests

Known Bezier fixtures, topology invariants, continuity, insert preserves curve, reverse twice identity, join/break, self intersections, extreme coordinates, randomized fuzz and undo/redo.