# G022 — Production RenderScene & Primitive Renderer

# Goal

Implement production backend foundation capable of rendering first vertical slice.

# Depends

G021, G018, G003.

# Primary

renderer-engineer.

# Skills

rendering-2d, shaders, scene-graph, lang-cpp, performance-native.

# Deliverables

RenderScene types; renderer resource IDs; paths/rectangles basic fill; raster image quad placeholder; transforms; clips baseline; group opacity; clear/background; frame scheduling integration; offscreen render for goldens; CPU/reference path where planned.

# Acceptance

Same rectangle/image scene renders on interactive canvas and offscreen test. Device/swapchain recreation works. No document/backend concrete types cross boundary.

# Tests

Golden simple scenes, transforms/clips, DPR, resize, device loss injection, leak/resource counts and frame timing instrumentation.

# Non-goals

Full blend/effects/text, advanced tessellation.