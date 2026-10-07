# 28.2 — Advanced Design Roadmap: Mesh Gradients, Pattern Systems, Warp, Brushes & CAD Interchange

# Mesh gradients

Post-V1 Planned. Canonical mesh control points/patch topology, color interpolation space, on-canvas node/handle editing and SVG/PDF degradation strategy required.

# Pattern system

Reusable pattern resources with tile geometry, transform, color overrides and live preview. Avoid raster-only representation when vector pattern source exists.

# Advanced warp

Envelope meshes, arc/flag/fisheye presets, freeform vector warp and bend-on-path. Canonical WarpNode should remain renderer-independent.

# Advanced vector brushes

Art/scatter/pattern brushes with resource dependencies, stretch/repeat rules, corner treatment and Expand fidelity.

# Repeater/procedural layout

Optional live repeat/grid/radial duplication nodes with parameterized instances, distinct from Symbol semantics.

# CAD interchange

DXF/DWG roadmap with units, arcs/splines/layers and model/paper space. Not a reason to introduce CAD-centric constraints into core document.

# Status

Each enters product only via dedicated capability spec/Goal and not through “small enhancement” to basic tools.