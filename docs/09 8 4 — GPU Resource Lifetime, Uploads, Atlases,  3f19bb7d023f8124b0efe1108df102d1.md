# 09.8.4 — GPU Resource Lifetime, Uploads, Atlases, Pipeline Cache, Synchronization & Device Loss

# Device abstraction

RenderDevice owns queues, capabilities, formats, pipeline cache and resource factories. RenderSession owns view/swapchain context.

# Buffers/textures

Resources use generation-safe handles; deletion deferred until GPU fence guarantees no use. Canonical code never stores raw backend handles.

# Upload

Staging ring/buffer pools batch geometry/tile uploads. Upload budget per frame prevents 8k image opening from starving interaction; visible priority first.

# Path meshes

Tessellation cache key includes path geometry revision, stroke/fill params, transform scale class/quality if tessellation is scale dependent.

# Glyph atlas

Separate monochrome/color glyph atlas classes; LRU pages; glyph key font face/variation/glyph/render mode/size scale. Text layout independent from atlas residency.

# Pipeline cache

Key shader/effect/blend/format variants. Prewarm common pipelines; compile asynchronously where backend permits. Variant count monitored.

# Synchronization

Explicit ownership barriers handled backend. CPU does not block for GPU every frame. Readback only for actual feature (picker/export/test) and asynchronously when possible.

# Device loss

Invalidate all backend handles/pipelines, retain CPU/document/derived source, recreate device/session, progressively repopulate visible resources. User sees transient recovery notification only if perceptible.

# Capabilities

Unsupported GPU feature routes alternate shader/pass/CPU reference; no document feature becomes corrupt because device lacks optional capability.

# Tests

Forced device loss, resource churn, atlas eviction, huge upload, repeated window recreation, multi-window sharing and leak tracking.