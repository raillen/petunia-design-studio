# 18.18 — Hand & Zoom Tool Specifications

# Hand

ToolId ptnd.tool.hand. Space temporary override. Drag changes viewport pan only.

# Zoom

ToolId ptnd.tool.zoom. Click zoom in, modifier zoom out, drag marquee fit, wheel/pinch shared viewport service.

# View state

Pan/zoom live outside document/history. Optional separate view-history tracks Previous Zoom without affecting undo.

# Anchoring

Wheel/pinch zoom keeps pointer/gesture centroid stable in document coordinates within numeric tolerance.

# Bounds

Viewport service defines min/max zoom and safe large-coordinate handling. Fit commands calculate Surface/Selection/All bounds with padding tokens.

# Temporary tool

InputModeStack remembers previous tool. Temporary Hand cannot interrupt a non-suspendable active gesture; status indicates why.

# Commands/actions

No document Commands. Actions ZoomIn, ZoomOut, Zoom100, FitSelection, FitSurface, FitAll, PreviousZoom.

# Tests

Fractional DPR, mixed-DPI move, extreme zoom, multiple views and temporary-tool restoration.