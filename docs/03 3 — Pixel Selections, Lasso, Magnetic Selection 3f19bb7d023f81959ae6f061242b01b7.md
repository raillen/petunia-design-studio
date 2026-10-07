# 03.3 — Pixel Selections, Lasso, Magnetic Selection, Selection Brush & Refine Workflow

# Selection model

PixelSelection is session/document editing state with tile mask representation and revision. It can be promoted into a canonical mask/channel by command.

# Combine

New, Add, Subtract, Intersect are common modes across marquee/lasso/brush/magic-like tools.

# Marquee

Rectangle, ellipse, row/column. Screen interaction remains smooth at any zoom; selection shape evaluates in document/pixel coordinate space.

# Lasso

Freehand, polygon and magnetic. Polygon supports backtrack; magnetic uses native edge analysis and visible anchors/confidence.

# Selection brush

Paint mask with edge-aware mode optional. Add/subtract shortcut clearly reflected in cursor/HUD.

# Feather

Selection can carry feather/smoothing treatment or command can bake mask; semantics chosen consistently so repeated operations do not compound unexpectedly.

# Refine

Dedicated refine state receives source selection and current composite snapshot. Controls: radius, feather, smooth, contrast/decontaminate if implemented, edge brush and preview overlay.

Outputs: Selection, Pixel Mask, New Layer with Mask or other explicit target.

# Quick mask

Optional view/edit mode converts selection to overlay and paint target without destroying source until commit.

# Commands

Selection combine operations are undoable according selection-history policy; conversion to mask/channel definitely canonical.

# Performance

Edge/refine jobs cancellable and revision-safe. Preview may be lower quality; Apply full quality.

# Tests

Hair/edge fixtures, alpha boundaries, high-res image, combine invariants, feather at tile edges and refine cancel.