# 09.8.1 — RenderScene IR, Display List, Draw Commands & Resource Handles

# Goal

Define a renderer-neutral intermediate representation that is rich enough for Petunia semantics and stable enough to support multiple GPU/CPU backends.

# Frame input

RenderFrameRequest {

DocumentRevision revision;

Surface/view set;

ViewTransform;

viewport/device extent;

quality profile;

proof/display transform;

overlay descriptors;

}

# RenderScene

Immutable evaluated snapshot composed of ordered render nodes. Recommended node vocabulary:

```
BeginSurface
BeginGroup / EndGroup
PushClipPath / PushClipRect / PopClip
PushMask / PopMask
DrawPath
DrawImage
DrawGlyphRun
ApplyEffectChain
CompositeOffscreen
BeginIsolation / EndIsolation
```

Exact representation can be flat display list plus side tables; semantics are more important than class hierarchy.

# Resource handles

PathGeometryHandle, ImageHandle, GlyphRunHandle, Brush/PaintHandle, EffectHandle refer to immutable derived resources with generation/revision. Backend never dereferences DocumentStore objects directly.

# DrawPath

geometry handle, transform, appearance snapshot, clip/mask state, opacity/blend. Multiple fill/stroke appearance entries may expand into ordered draw operations while preserving object-level isolation semantics.

# DrawImage

image/tile resource, source rect, transform, sampling policy, opacity/blend, color transform reference.

# DrawGlyphRun

font face/glyph IDs/positions, transform, fill/stroke appearance and color-space context. Renderer is not responsible for shaping.

# Determinism

Given same RenderScene and backend reference mode, order and semantic output are deterministic. Hash/debug serialization of scene is available in dev/tests.

# Incremental

Scene builder can reuse unchanged immutable nodes/resources keyed by canonical/derived revisions; scene identity never becomes document truth.