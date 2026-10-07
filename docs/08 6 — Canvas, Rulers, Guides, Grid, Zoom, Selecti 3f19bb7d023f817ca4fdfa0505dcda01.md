# 08.6 — Canvas, Rulers, Guides, Grid, Zoom, Selection, Snapping & HUD

# Canvas

Região dominante. RenderSession C++ projeta documento; CanvasWidget gerencia viewport/input apenas.

# Background/Surface

Neutral workspace background; Surface branca/transparent conforme document. Mostrar boundary, optional shadow, bleed, margins, columns e label outside bounds.

# Rulers

Top/left toggle, units, origin, guide drag, coordinate highlight. Right-click troca units. Double click origin resets.

# Guides

Normal/hover/selected/locked/snap-candidate. Drag cria/move; Alt duplicates; Delete removes; numeric edit via Properties/Guide UI. Locked guide não captura drag mas pode snap conforme setting.

# Grids

Rectangular, pixel, baseline e perspective. Cada família tem spacing/subdivisions/origin/display/snap. Perspective é subsystem próprio.

# Zoom

Wheel/pinch anchored near pointer; +/−; 100%; Fit Selection; Fit Surface; Fit All; Previous Zoom; numeric field. Avoid float precision drift em documentos gigantes.

# Pan

Space-drag, middle mouse optional, trackpad gestures. Temporarily overrides cursor/tool without ending current operation.

# Selection visuals

Bounding box, handles, pivot, node/path outlines. Dual-tone stroke garante contraste sobre qualquer artwork. Handles mantêm hit size screen-space estável.

# Snapping

Candidates: nodes, tangents, edges, centers, bbox, guides, grid, margins, text baselines, spacing/distribution. Engine C++ retorna ranked SnapCandidate; UI desenha indicator + measure label. Candidate clutter é reduzido por priority/hysteresis.

# HUD

Transform W/H/angle, brush size/hardness, live shape params, quick boolean/selection actions. HUD evita pointer hotspot, Esc dismisses e controles são keyboard focusable.

# Overlay order

content -> proof/pixel preview -> grids/page guides -> snapping/measure -> selection/nodes -> active tool preview -> HUD/drag affordances.