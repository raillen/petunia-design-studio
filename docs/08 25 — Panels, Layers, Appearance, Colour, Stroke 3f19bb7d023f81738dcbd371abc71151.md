# 08.25 — Panels, Layers, Appearance, Colour, Stroke, Transform, Assets & Symbols UX Contract

# Panel family consistency

Every panel supports dock/floating state, keyboard focus, searchable content if scale warrants, empty/loading/error and provider disappearance.

# Layers

Tree must show enough type/status without icon noise. Drop zones distinguish reordering, nesting, clipping and masking.

# Appearance

Multiple fills/strokes/effects are first-class. Selecting an entry changes target of Color/Stroke/gradient tool visibly.

# Colour

Model/profile visible when professional color matters. Fill/Stroke target always visible. Spot/global swatch state has non-color badge/text.

# Stroke

Graphical cap/join controls plus numeric width/dash; pressure graph keyboard-accessible through table/numeric alternative.

# Transform

Numeric precision never requires canvas drag. Coordinate-space/anchor is explicit.

# Assets/Symbols

Thumbnail grid with search/categories; drag-place plus keyboard/context Place. Symbols distinguish definition vs instance and override count.

# Cross-panel sync

Panels subscribe to Selection/Property snapshots and ChangeSets. Updating one panel cannot create feedback loops or duplicate Commands.