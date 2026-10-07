# 18.25 — Marquee Selection Tools Specification

# Tools

Rectangle, Ellipse, Single Row and Single Column marquee variants under selection tool group.

# State

Idle → DraggingShape → PreviewSelection → Commit/Cancel.

# Combine

New, Add, Subtract, Intersect visible in context. Modifier shortcuts may temporarily change combine mode but HUD/status reflect current mode.

# Geometry

Rectangle/ellipse created in pixel/document coordinates; Shift constrains square/circle; Alt center-out. Space can reposition current marquee if adopted.

# Feather/anti-alias

Context fields define selection edge semantics. Preview can show soft mask overlay plus ants threshold.

# Row/Column

One-pixel/document-pixel semantic thickness tied to target raster pixel grid, not screen pixel.

# Commands/state

Selection is session editing state with undo policy; conversion to mask/channel is canonical command.

# Tests

All combine algebra, feather, transformed raster target, extreme zoom and keyboard numeric selection via Select dialog/action.