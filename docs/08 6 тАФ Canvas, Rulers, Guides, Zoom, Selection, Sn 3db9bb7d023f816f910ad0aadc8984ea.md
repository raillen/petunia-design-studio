# 08.6 — Canvas, Rulers, Guides, Zoom, Selection, Snapping & On-Canvas HUD

# Canvas role

Canvas is the dominant visual area and receives the quietest surrounding chrome. It is a viewport over the canonical document/evaluated scene; it does not own document state.

# Workspace background

Neutral adaptive tone distinct from artboards/surfaces. User can choose light/dark/checkerboard variants where useful, but background must not be mistaken for export content.

# Surfaces/artboards

Each Surface displays:

- page/artboard boundary;
- optional shadow only for spatial separation;
- bleed region visualization;
- margins/columns/guides;
- labels outside bounds when enabled;
- selection state when Surface tool active.

# Rulers

Top and left rulers can be toggled. Features:

- document units;
- origin indicator;
- drag origin reset;
- guide creation by ruler drag;
- cursor coordinate highlight optional;
- high-DPI crisp ticks;
- context menu for units/origin options.

# Guides

Guide states:

- normal;
- hovered;
- selected;
- locked;
- snapping candidate;
- hidden.

Guides can be dragged, duplicated, numerically edited and locked. Core guide creation, movement, numeric editing, locking and visibility are **V1 Required**. A dedicated named Guide Manager is a **Post-V1 Candidate** unless promoted; if present it edits the same guide model and Actions rather than owning a second guide store.

# Grid

**V1 Required grid/overlay families:**

- rectangular document grid;
- pixel grid for pixel-precision contexts;
- baseline grid for layout/text workflows;
- perspective grid as the visual/editor surface of the accepted perspective subsystem in 10.8. Perspective grid is not implemented as a variant of the ordinary rectangular-grid data model merely for UI convenience.

Settings include spacing, subdivisions, origin, color/opacity and snap behavior.

# Zoom

Supported actions:

- scroll/pinch zoom centered near pointer where intuitive;
- zoom in/out shortcuts;
- 100%;
- Fit Selection;
- Fit Surface;
- Fit All;
- Previous Zoom;
- numeric zoom field/status control.

Zoom animation short and optional; precision workflows may prefer immediate zoom.

# Pan

Space-drag as standard; middle mouse optional; trackpad gesture. Cursor changes to hand. While panning, tool operation state must be preserved.

# Selection visuals

Object selection uses:

- bounding box;
- handles;
- center/origin marker when relevant;
- node/path highlight for node editing;
- readable contrast over any artwork using dual-tone outline if needed.

Selection color configurable for accessibility but defaults to Aubrieta canvas selection accent independent of UI accent if contrast requires.

# Transform handles

Resize handles scale visually independent of zoom within practical limits. Rotation affordance appears at predictable offset. Modifier keys support aspect lock, center scaling, duplication etc. Tooltip/HUD shows dimensions/angle during drag.

# Node editing

Nodes distinguish cusp/smooth/symmetric types by shape + state, not color alone. Bézier handles use thinner lines and larger hit areas than visible points. Overlapping nodes expose cycling/selection strategies.

# Snapping

SnapEngine feeds UI overlays. Possible targets:

- nodes;
- edges;
- midpoints;
- object centers;
- bounding boxes;
- guides;
- grid;
- margins/columns;
- text baselines;
- equal spacing/distribution.

On snap, show minimal temporary indicators and measurement labels. Multiple candidate clutter must be resolved by priority.

# Smart guides / measurements

Display only during relevant transformation. Labels use compact monospace/tabular numeric text. Distance arrows/lines disappear rapidly after operation.

# On-canvas context HUD

Small transient HUD may appear near selection/pointer for:

- transform dimensions;
- brush size/hardness;
- selection refine actions;
- shape live parameters;
- boolean quick actions.

HUD rules:

- never cover active pointer target if avoidable;
- movable only if it persists long enough to justify;
- keyboard navigation available for actionable controls;
- Esc dismisses transient HUD;
- advanced editing remains in Properties.

# Overlays

Possible overlays:

- grid/guides;
- margins/bleed;
- pixel preview;
- proof colors/gamut warning;
- mask overlay;
- transparency grid;
- selected object outlines;
- snapping;
- data merge field hints.

Overlay ordering must be explicit to prevent guides vanishing behind mask/proof overlays. Canonical semantic stacking from lower to higher is: document/render content → proof/pixel-preview transforms → noninteractive page/margin/bleed/grid overlays → guides/snapping/measurement overlays → selection/path/node handles → active-tool preview → transient HUD/drag/drop affordances. Mask/selection overlays may modify this order only through a documented tool-specific overlay layer; z-order is semantic metadata, not raw GPUI constants.

# Canvas context menu

Selection-sensitive. Empty-canvas menu includes Paste, Select All, View options, Insert/Place, Grid/Guide shortcuts. Object menu includes object operations. Avoid giant menu.

# Scroll/edge behavior

Drag operations auto-pan near viewport edges with acceleration. Canvas should support very large document extents without integer precision glitches; UI labels remain stable.

# Cursor system

Semantic cursors: select, move, node, pen, crosshair, text I-beam, rotate, scale directions, eyedropper, brush, eraser, zoom, hand, forbidden, copy. Cursor and tool icon state must remain synchronized.