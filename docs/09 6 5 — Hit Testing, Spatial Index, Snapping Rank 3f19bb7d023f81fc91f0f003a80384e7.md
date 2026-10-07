# 09.6.5 — Hit Testing, Spatial Index, Snapping Ranking, Hysteresis & Overlay Contract

# Spatial index

Per-document/Surface derived R-tree/BVH candidate over conservative world bounds. Incremental ChangeSet updates affected entries.

# Hit query

Input:

document point, screen tolerance, viewport transform, desired capabilities, hidden/locked/include mode, active tool.

Output ordered HitCandidate[] with ObjectId, component kind, sub-ID, distance_px, z-order, score metadata.

# Component kinds

FillInterior, Stroke, Node, Handle, Segment, BoundingHandle, TextGlyph/Frame, Guide, Surface, MaskHandle.

# Ranking

Tool-specific priority + distance + z-order + explicit cycling state. Node Tool prioritizes selected path nodes/handles; Move Tool prioritizes visible topmost object unless select-through.

# Snapping candidates

Node, endpoint, midpoint, curve nearest/extrema, tangent, center, bbox edge/corner, guide, grid, margin/column, baseline, equal-spacing, angle.

# Threshold

User tolerance in logical screen px converted through viewport. Each category can apply multiplier but defaults are centrally configured.

# Hysteresis

Once snapped, keep target until pointer exits release threshold > acquisition threshold or clearly better candidate exceeds score margin. Prevents flicker between nearby points.

# Constraints

Axis/angle constraint and snapping are solved together. Snap engine returns proposed semantic transform/point, not just visual hint.

# Overlay descriptor

SnapLine, SnapPoint, Measurement, EqualGap, Label with document/screen anchors, target IDs and accessible text. UI renders; engine remains toolkit-free.

# Performance

Broad-phase under few milliseconds for 100k+ objects. Batch hit/snap queries can reuse spatial neighborhood during drag.

# Tests

Dense overlaps, rotated objects, high/low zoom, equal gaps, hysteresis, category toggles, hidden/locked, selection cycle, accessibility descriptor.