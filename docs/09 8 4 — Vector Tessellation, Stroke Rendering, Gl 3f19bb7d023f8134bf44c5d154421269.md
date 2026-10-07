# 09.8.4 — Vector Tessellation, Stroke Rendering, Glyph/Image Caches & GPU Resource Lifetime

# Vector rendering

Path geometry can be tessellated CPU-side, GPU path-rendered or delegated to backend library. Document semantics remain path-level.

# Tessellation cache

Key includes evaluated PathGeometry revision, fill rule, transform scale bucket/tolerance and backend-specific representation. Translation-only transforms should reuse geometry where possible.

# Curves

Tessellation error expressed in device-space pixels; zoom changes can trigger quality bucket rebuild. Avoid retessellating every small zoom delta.

# Strokes

Renderer and ExpandStroke share StrokeEvaluator semantics. GPU may render analytic stroke if visually equivalent; expanded reference geometry provides oracle for difficult caps/joins/dashes.

# Glyph cache

Glyph atlas keyed by FontFaceId, variation coordinates, glyph ID, size/render mode/subpixel policy. Eviction bounded by VRAM; layout positions remain independent.

# Image cache

Decoded tile -> color-transformed/upload-ready representation -> GPU texture tile/page. Residency tracks base generation and display/color transform generation.

# Resource lifetime

BackendResourceHandle owns device resource and destruction fence/deferred queue. Document deletion only drops semantic references; GPU resource may survive until frame completion.

# Atlas fragmentation

Use paged atlases or texture arrays where backend permits. Large images remain tiled textures instead of forcing atlas.

# Device loss

All GPU cache entries disposable; CPU/evaluated sources enough to rebuild.

# Diagnostics

Per-cache bytes/count/hit/miss/eviction/rebuild time surfaced in dev panel.