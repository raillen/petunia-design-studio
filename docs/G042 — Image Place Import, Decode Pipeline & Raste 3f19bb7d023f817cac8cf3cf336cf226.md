# G042 — Image Place/Import, Decode Pipeline & Raster Layer Rendering

# Goal

Place/import professional raster resources efficiently.

# Depends

G041, 04.9, IO framework, renderer.

# Primary

engine/editor engineer.

# Deliverables

image sniff/adapter; libvips/codecs path; linked/embedded PlacedResource; profile/metadata extraction; lazy/tiled decode; RasterLayer RenderScene integration; missing/relink state.

# Acceptance

High-res PNG/JPEG/TIFF/WebP fixture places quickly, preserves profile/link metadata and progressively renders without duplicate full buffers.

# Tests

huge dimensions, alpha, profiles, EXIF orientation, malformed files, missing links and performance.