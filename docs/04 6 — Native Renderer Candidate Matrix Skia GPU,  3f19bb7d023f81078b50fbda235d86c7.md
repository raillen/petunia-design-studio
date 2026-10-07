# 04.6 — Native Renderer Candidate Matrix: Skia GPU, Dawn/WebGPU, Qt RHI & Custom Backends

# Candidates

**Skia GPU** — mature 2D rendering primitives, text/vector ecosystem, native GPU backends.

**Dawn/WebGPU** — portable modern GPU abstraction with explicit pipeline/resource model.

**Qt RHI** — useful Qt-integrated abstraction candidate for presentation/hosting; evaluate suitability for full creative renderer.

**Custom abstraction over Vulkan/Metal/D3D** — maximum control, highest engineering cost.

# Hard constraints

Windows/Linux/macOS; offscreen rendering; device loss; high DPI; large textures; vector paths; clips/masks; blend/effects; color-transform hooks; headless tests; deterministic export path; acceptable binary/license footprint.

# Evaluation principle

The winner is selected by workload evidence, not preference. 15.3 owns falsification benchmarks.

# Architecture protection

RenderScene and RenderBackend isolate document/evaluation semantics. Choosing Skia must not make SkPath document truth; choosing Dawn must not make WGSL layout document truth.

# CPU path

A CPU renderer/reference path is still needed for deterministic testing/headless fallback even if GPU backend dominates interaction.

# Qt integration

Qt provides window/native surface/lifecycle events. Renderer owns content frame resources. QWidget paint events merely schedule/present native rendering.

# Future backends

Backend registry can add software/experimental backend without changing public document/plugin APIs.