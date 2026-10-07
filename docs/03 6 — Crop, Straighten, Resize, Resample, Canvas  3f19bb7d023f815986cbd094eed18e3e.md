# 03.6 — Crop, Straighten, Resize, Resample, Canvas/Surface Size & Image Transform Semantics

# Crop

Default crop should be nondestructive where feasible: alter view/Surface crop metadata or clipping bounds while preserving underlying pixels.

# Trim/destructive crop

Separate action discards pixels outside chosen extent and warns when irreversible beyond undo/history.

# Straighten

User draws horizon/angle or rotates crop overlay. Final operation composes transform and crop semantics without hidden resample until necessary.

# Resize image

Changes raster pixel dimensions with chosen resampling filter, aspect lock and physical DPI metadata policy.

# Resize canvas/Surface

Changes document/Surface bounds without resampling pixels; anchor controls content placement.

# Resample

Explicit algorithm list with high-quality defaults and preview. Downsample/upscale behavior benchmarked against reference images.

# Rotate/flip

Lossless metadata/transform when possible; destructive pixel rotation only on explicit raster operation.

# GUI

Crop overlay, aspect presets, straighten control, dimensions, DPI, resample method and live/destructive label.

# Export distinction

Export scaling does not mutate document; document resize command does.

# Tests

Odd sizes, transparency, 16-bit, profiles, rotated crop, repeated nondestructive edit and save/reopen.