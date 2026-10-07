# 09.8 — Render Scene, GPU Backend, Compositor, Shaders & Device Loss

# Boundary

Document/evaluation produces renderer-neutral RenderScene/DisplayList. Backend consumes immutable frame snapshot.

# Backend ADR

V1 evaluates Skia GPU, Dawn/WebGPU and native abstraction candidates. Technology Decision Agent runs falsification benchmarks across Windows/Linux/macOS. Choice cannot leak into document APIs.

# Render pipeline

visible Surface cull -> scene flatten -> clip/mask plan -> vector/raster/text resources -> effect passes -> blend/composite -> proof/pixel-preview transforms -> overlays -> present.

# Batching

Group compatible draws while preserving paint semantics. Texture atlases/caches bounded. Avoid allocation churn per frame.

# Shaders

Host-device structs explicit, offline/precompiled where possible, reflection validated. Shader variants bounded to avoid pipeline explosion.

# Color

Compositor works in declared working/compositing space policy. Display conversion is final view transform, not document mutation.

# Quality

Interactive preview may lower filter resolution/tessellation under quality policy; idle/final redraw converges. Export renderer uses explicit final quality.

# Device loss

Backend reports loss -> drops GPU caches -> recreate device/swapchain -> rebuild from canonical/derived CPU state. Document remains safe.

# CPU fallback

At minimum headless/export/testing and unsupported GPU path retain correct CPU renderer for core formats/features, even if slower.

# Diagnostics

Frame timings, passes, draw counts, upload bytes, VRAM, cache stats and backend errors accessible in dev panel/crash bundle.

[09.8.1 — RenderScene IR: Draw Primitives, State Stack, Clips, Masks, Groups & Stable Resource IDs](09%208%201%20%E2%80%94%20RenderScene%20IR%20Draw%20Primitives,%20State%20Sta%203f19bb7d023f81bf851af54d13043c8d.md)

[09.8.2 — Compositor Mathematics: Premultiplied Alpha, Blend Modes, Isolation, Pass-Through & Working Space](09%208%202%20%E2%80%94%20Compositor%20Mathematics%20Premultiplied%20Alph%203f19bb7d023f813ba570c29e08f958f3.md)

[09.8.3 — Frame Graph, Offscreen Pass Planning, Effect ROI, Transient Targets & Partial Redraw](09%208%203%20%E2%80%94%20Frame%20Graph,%20Offscreen%20Pass%20Planning,%20Eff%203f19bb7d023f816499aee6ed6390d40c.md)

[09.8.4 — GPU Resource Lifetime, Uploads, Atlases, Pipeline Cache, Synchronization & Device Loss](09%208%204%20%E2%80%94%20GPU%20Resource%20Lifetime,%20Uploads,%20Atlases,%20%203f19bb7d023f8124b0efe1108df102d1.md)

[09.8.5 — Vector Tessellation, Raster Tiles, Text Rendering, Antialiasing & Pixel-Preview Semantics](09%208%205%20%E2%80%94%20Vector%20Tessellation,%20Raster%20Tiles,%20Text%20R%203f19bb7d023f8159b7caed3ca00f3687.md)

[09.8.1 — RenderScene IR, Display List, Draw Commands & Resource Handles](09%208%201%20%E2%80%94%20RenderScene%20IR,%20Display%20List,%20Draw%20Comman%203f19bb7d023f817483c6f98c158f2a25.md)

[09.8.2 — Compositor Mathematics: Premultiplied Alpha, Blend Modes, Groups & Isolation](09%208%202%20%E2%80%94%20Compositor%20Mathematics%20Premultiplied%20Alph%203f19bb7d023f81bbb68fe8f52c6b9cb2.md)

[09.8.3 — Frame Graph, Offscreen Passes, Effect ROI, Transient Textures & Scheduling](09%208%203%20%E2%80%94%20Frame%20Graph,%20Offscreen%20Passes,%20Effect%20ROI%203f19bb7d023f8115b290eca33d067a0e.md)

[09.8.4 — Vector Tessellation, Stroke Rendering, Glyph/Image Caches & GPU Resource Lifetime](09%208%204%20%E2%80%94%20Vector%20Tessellation,%20Stroke%20Rendering,%20Gl%203f19bb7d023f8134bf44c5d154421269.md)

[09.8.5 — Renderer Quality Profiles, Pixel Preview, Proofing, Device Loss & CPU Reference](09%208%205%20%E2%80%94%20Renderer%20Quality%20Profiles,%20Pixel%20Preview,%203f19bb7d023f81579aecd0877b0ffbbd.md)