# 18.14 — Knife / Scissors Tool Specification

# Knife

ToolId ptnd.tool.knife. Gesture defines cutter path/line through eligible vector objects. Native engine computes intersections and reconstructs topology.

# Modes

Topmost/selected/all-selected scope; optional straight/freehand cutter; close resulting shapes only if explicit.

# Preview

Affected objects and intersection points highlighted before/while cutting. Invalid/no-hit gesture commits nothing.

# Scissors

ToolId ptnd.tool.scissors. Click nearest path location/node and split/open contour. Existing node snap stronger than segment insertion.

# Commands

KnifeCut, SplitPathAt, BreakPath.

# Style

Cut results retain source appearance deterministically; newly exposed open ends use stroke semantics naturally.

# Accessibility/tests

Scissors supports keyboard candidate cycling/numeric path operations where possible. Test tangencies, multiple intersections, closed/open contours, self-intersections and undo.