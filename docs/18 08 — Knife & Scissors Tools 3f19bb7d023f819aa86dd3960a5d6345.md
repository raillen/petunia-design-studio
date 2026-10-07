# 18.08 — Knife & Scissors Tools

# Knife

Freehand or straight cut path over selected/topmost/all eligible vectors.

# Knife states

Idle -> DrawCut -> IntersectionPreview -> Commit/Cancel. Preview marks affected objects and cut intersections.

# Scissors

Hover path -> exact nearest cut marker -> click. Existing node reused; segment click inserts exact node then breaks.

# Options

Topmost/selected/all; close resulting shapes where valid; separate contours into objects; style inheritance.

# Topology

GeometryEngine performs robust intersection and splitting. Surviving NodeIds preserved; new cut nodes get provenance.

# Appearance

Result pieces retain source appearance/transform. Masks/live constructs follow explicit split policy or tool is disabled when semantics cannot be preserved.

# Commands

KnifeCut, BreakPathAt, SplitContoursToObjects.

# Accessibility

Path/object can be chosen through Layers; numeric segment parameter/selected node break actions exposed where useful.

# Tests

Tangency, self-intersection, multiple contours, exact-node cut, no-hit, style/mask preservation, save/undo.