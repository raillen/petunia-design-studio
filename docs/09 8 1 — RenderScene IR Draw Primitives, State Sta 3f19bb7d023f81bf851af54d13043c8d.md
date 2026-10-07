# 09.8.1 — RenderScene IR: Draw Primitives, State Stack, Clips, Masks, Groups & Stable Resource IDs

# Goal

Define a backend-neutral immutable intermediate representation produced from evaluated document state.

# Scene header

revision, view transform, target size/DPR, quality tier, color pipeline descriptor, Surface visibility.

# Primitive families

DrawPath, DrawImage/TileSet, DrawGlyphRun, DrawMesh/gradient if supported, BeginGroup/EndGroup, PushClip/PopClip, PushMask/PopMask, ApplyEffectChain, CompositeSurface, DebugOverlay hooks.

# State

Transform, opacity, blend mode, clip stack, mask stack, color space/working context and paint/appearance references are explicit. Backend cannot infer semantics from widget state.

# Resources

Scene refers to stable RenderResourceId objects: PathMeshKey, ImageResourceKey, GlyphAtlasKey, GradientKey, EffectKernelKey. Resource manager maps keys to backend objects.

# Groups

Group node declares isolation/pass-through, opacity/blend and bounds. Isolation forces offscreen when needed; pass-through affects descendant compositing according documented semantics.

# Clips

Vector clips are geometry coverage; nested clips intersect. Clip antialiasing matches mask coverage policy.

# Masks

Mask node includes coverage source, transform, invert and composition operation. Luminosity masks are explicit conversion/effect nodes.

# Ordering

RenderScene preserves canonical paint order. Optimization may batch only when result mathematically unchanged.

# Serialization

RenderScene is transient/not PTND. A debug textual dump may be produced for tests/diagnostics.

# Tests

Scene generation snapshots for representative docs, stable ordering, clips/groups/masks, no Qt/backend types.