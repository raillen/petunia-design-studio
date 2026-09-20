# 10.8 — Perspective, Projective Transform, Perspective Grid, Warp & Envelope

# Separate concepts

Aubrieta never conflates projective transform with freeform warp. `ProjectiveTransform` is homography; `PerspectivePlane/Grid` is editing aid/constraint; `Warp/Envelope` is deformation.

# Projective transform

Apply to vector/text/raster subtree non-destructively when possible. Four-corner handles map source quad to destination quad. Invalid/self-crossed quadrilateral gets constrained/rejected with clear preview.

# Perspective grid

One-, two- and three-point Perspective Grid modes are **V1_REQUIRED**, with horizon, vanishing points and active planes. Objects can snap/project to the active plane but remain ordinary document objects with modifier/transform, not permanently owned by the grid.

# Editing

Move vanishing points, horizon, origin/plane handles with visual measurements. Lock grid. Switch active plane without losing object data.

# Warp/Envelope

Mesh/envelope control points deform evaluated geometry/raster. Live modifier stores source + control lattice/mode. Bake commits per object type (path expansion or rasterization as appropriate).

# Text

Projective text can remain semantic text with transform until backend/export requires expansion/rasterization. Warp may preserve text modifier but editing text re-evaluates deformation.

# Raster

Photo content uses same projective modifier/compositor path, with filtering/resampling quality controls independent from canonical transform.

# Tests

Near-singular homographies, flipped corners, huge perspective, nested transforms, text editing under transform, raster quality, export degradation, undo/cancel.

# Implementation contract — V1

## Scope status

**V1_REQUIRED:** non-destructive four-corner projective transform for vector/text/raster subtrees; one-, two- and three-point Perspective Grid; plane snapping/projection; baseline Warp/Envelope modifier with editable lattice/control handles; explicit Bake/Expand/Rasterize commands where needed.

Advanced mesh-warp modes beyond the accepted baseline are `POST_V1_CANDIDATE` unless separately specified.

## Projective modifier semantics

A projective transform is a typed non-destructive modifier with source bounds/space and a validated homography/destination quadrilateral. Canonical document data stores the semantic transform parameters, not resampled raster output or flattened path geometry.

Evaluation rejects non-finite matrices and near-singular mappings under a documented determinant/condition threshold. Self-crossing destination quads are not silently accepted; the tool constrains or reports invalid geometry while preserving the last valid preview.

## Editing state machine

`Idle → TargetSelected → HandleDrag/QuadMove → ValidPreview | InvalidPreview → Commit | Cancel`.

Begin captures source IDs, revision, source quad, transform stack and pivot/plane context. Commit revalidates source revision. Cancel restores the exact pre-interaction modifier state.

## Transform stacking

ProjectiveTransform has a documented position relative to ordinary affine transform, warp/envelope and Appearance/effects. UI reorder or Bake actions must correspond to the evaluation order specified by the modifier/evaluation architecture; agents may not arbitrarily multiply matrices in toolkit code.

## Perspective Grid

The grid is document/session semantic aid according to persistence scope chosen by its descriptor, with stable GridId where document-persistent. V1 supports 1/2/3-point modes, horizon, vanishing points, origin and active plane. Grid editing uses semantic constraints so moving a vanishing point updates derived lines without modifying objects already placed on the plane.

Objects may snap/project to an active plane via explicit commands/modifiers; they are not owned by the grid and remain editable if the grid is hidden/deleted.

## Perspective placement

“Attach/project to plane” stores enough semantic relation to re-evaluate if live mode is selected. A bake/static placement variant produces ordinary object transforms. UI must distinguish live plane relation from one-time snap.

## Warp/Envelope

Warp/Envelope is a separate typed deformation modifier with source object/reference, lattice/control points and mode. Editing source text/path reruns deformation. Lattice resolution and boundary behavior are versioned parameters. Invalid/folded lattice regions may be allowed only if the chosen mode defines them; otherwise report invalid preview.

## Text behavior

Text remains canonical text while projective/warp modifiers are live. Caret/editing coordinates are mapped through inverse transform where stable; if inverse mapping becomes numerically invalid, text editing is disabled with a clear reason until geometry is repaired. Export may outline/rasterize only through explicit degradation policy.

## Raster behavior

Raster projective/warp evaluation uses resampling quality as render/export context, not stored destructive pixels. Final rasterization is an explicit command. Preview quality may be reduced during drag but final display/export uses required quality/color management.

## Snapping and input

Perspective handles/grid lines integrate with the shared Snap service using semantic candidate types. Physical modifier keys are mapped by keymap to intents; domain code never hard-codes Shift/Alt/Ctrl.

## Automation/plugins

MCP/Lua can create/edit modifiers and grids through typed parameters and stable IDs. Plugin scripting cannot submit arbitrary GPU shader deformation; custom deformation requires a registered semantic effect/modifier capability and safety/version contract.

## Required tests

Add near-singular threshold, invalid/self-crossed quad recovery, nested affine+projective+warp ordering, stale revision during drag, plane grid deletion with attached/live objects, inverse text editing, raster preview/final quality difference, explicit bake/rasterize undo, save/reopen and UI↔MCP parity.