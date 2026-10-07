# 06.3 — Skia Prior Art: 2D Rendering, Paths, Images, Text, Filters & Backend Isolation

# Why study Skia

Skia is a mature cross-platform 2D graphics library covering paths, images, text, antialiasing, transparency, filters and multiple rendering/output mechanisms.

# Adopt

Use as renderer candidate because it already represents many 2D operations Petunia needs.

Keep Skia strictly behind RenderBackend/adapter: SkPath/SkPaint/SkImage cannot become PTND canonical model.

Evaluate existing path effects, filters, shaders and color management before reimplementing equivalent low-level rendering machinery.

# Architecture mapping

Petunia VectorPath -> renderer adapter path representation.

Color/Appearance -> backend paint/shader.

RenderScene clips/groups/effects -> Skia canvas/surface/pass plan when chosen.

PTND/export semantics remain Petunia-owned.

# Risks

Build footprint, backend evolution, exact GPU API path, integration with custom effect graph and need for deterministic CPU/reference behavior.

# Source

- [Skia documentation](https://skia.org/docs/)
- [Skia user documentation](https://skia.org/docs/user/)

# Decision rule

Skia's feature breadth is an advantage only if benchmark/integration evidence from G020/G021 beats alternatives without contaminating domain architecture.