# 10.8 — Perspective, Projective Transform, Perspective Grid, Warp & Envelope

# Distinct concepts

**Projective Transform** changes object coordinates by homography.

**Perspective Grid/Plane** is construction/snap infrastructure.

**Warp/Envelope** is nonlinear deformation.

UI may group them, but core models remain separate.

# Perspective Grid

One/two/three-point construction modes as product scope. Grid stores vanishing points/axes/horizon and display subdivisions. It is view/document construction metadata and a snapping provider.

# Perspective drawing

When plane active, creation tools can map local plane coordinates to document projective coordinates. Shapes/text remain semantic objects with ProjectiveTransform where representation allows.

# Projective Transform Tool

Four-corner quad handles + edge/pivot controls. Drag one corner previews homography; modifier constrains. Context reset/copy/paste plane, numeric matrix advanced. Invalid/degenerated quad rejected with visual warning.

# Text/vector

Prefer retaining editable source plus projective transform. Renderer evaluates path/text then transform. Export adapters preserve if representable or expand/rasterize with preflight.

# Raster perspective

Can be live transform node over raster source; sampling quality preview/final. Destructive apply explicit.

# Warp/Envelope

Canonical WarpNode references source + warp kind/mesh/parameters. Basic Arc/Bulge/Flag plus 2×2/3×3 envelope candidates. Control points on canvas; Properties provides numeric preset/strength.

# Mesh warp

Mesh nodes/handles are sub-selection. Moving nodes updates derived deformation; high-cost preview can use lower resolution. Commit one transaction.

# Bake

Bake Warp/Projective converts vector geometry/text/raster according target-specific destructive semantics and prompts/preflights if editability loss matters.

# Snapping

Perspective grid supplies ranked snap lines/points. Ordinary grid remains separate. Transform handles can snap to geometry/guides.

# Commands

CreatePerspectiveGrid, SetPerspectiveGrid, SetProjectiveTransform, ResetProjectiveTransform, AddWarp, SetWarpParameters, EditWarpMesh, BakeWarp.

# Tests

Homography inverse/degenerate cases, horizon extremes, text/vector editability, raster resampling, warp continuity, undo, export degradation and large object preview latency.