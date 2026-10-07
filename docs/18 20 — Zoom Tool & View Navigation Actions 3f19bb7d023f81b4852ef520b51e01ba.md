# 18.20 — Zoom Tool & View Navigation Actions

# Identity

ToolId ptnd.tool.zoom. Z candidate; common +/−/Ctrl+wheel/pinch actions share ViewportController.

# Interaction

Click zoom in around pointer; modifier click zoom out; drag marquee fits chosen document rectangle; wheel/pinch continuous anchored zoom.

# Actions

ZoomIn, ZoomOut, Zoom100, FitSelection, FitSurface, FitAll, PreviousZoom, SetZoom.

# Precision

Zoom represented double with bounded min/max derived from document/viewport sizes. Anchor point stays visually stable during zoom to avoid drift.

# History

View zoom is not document Undo. PreviousZoom keeps small view-navigation history separately.

# Pixel preview

100% can mean document pixel mapped to one screen logical/physical pixel depending view mode; UI labels semantics clearly.

# Accessibility

Zoom field, shortcuts and menu actions; announce level.

# Tests

Fractional DPI, huge Surface, pointer anchor, fit calculations, previous zoom and no document revision.