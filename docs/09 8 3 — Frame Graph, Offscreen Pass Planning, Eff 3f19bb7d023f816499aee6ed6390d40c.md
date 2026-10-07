# 09.8.3 — Frame Graph, Offscreen Pass Planning, Effect ROI, Transient Targets & Partial Redraw

# Render planning

RenderPlanner converts RenderScene into FrameGraph DAG:

resource uploads -> geometry/text preparation -> offscreen group/effect passes -> main composite -> display/proof transform -> overlays -> present.

# Pass

PassId, inputs, outputs, bounds/ROI, format/color space, clear policy, dependencies, quality tier and backend pipeline key.

# ROI

Effect declares input expansion function. Gaussian blur radius r requests halo based on sigma/kernel. Planner propagates damage/needed region backwards through effect graph.

# Partial redraw

Viewport damage from ChangeSet/tool overlay/view movement maps to affected screen/document region. If backend/swapchain permits retained/offscreen strategy, redraw minimal; otherwise scene culling still limits work. Correctness first.

# Transient resources

TexturePool allocates offscreen targets by size/format/sample count with lifetime intervals from FrameGraph. Alias only non-overlapping lifetimes.

# Tiling

Effects exceeding max texture/memory can execute tiled with halo and seam-free combination where algorithm supports.

# Quality tiers

InteractiveFast, InteractiveStable, FinalView, ExportFinal. Effect/tessellation implementation declares allowed approximation per tier and convergence rule.

# Cancellation/staleness

Background preparation keyed scene revision; stale mesh/effect result not inserted into newer frame unless content fingerprint still valid.

# Tests

ROI propagation, blur seams, nested effects, memory peak planning, partial damage, quality convergence and deterministic graph dump.