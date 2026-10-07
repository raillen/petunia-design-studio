# 09.8.5 — Vector Tessellation, Raster Tiles, Text Rendering, Antialiasing & Pixel-Preview Semantics

# Vector

Fill tessellation respects fill rule and transform. Stroke tessellation/evaluation matches StrokeEngine. Analytical/path rendering backend may replace triangulation behind same semantic output.

# Antialiasing

Coverage antialiasing definition is consistent for vector edges, clips and masks. MSAA/analytic alternatives must meet visual tolerance; no platform-random edge rule.

# Raster

Tile texture sampling honors source pixel center, filter mode, transform and color conversion. Zoomed-out uses mips; final export chooses defined resampler.

# Text

GlyphRun uses shaped glyph IDs/positions. Small UI-like text in artwork may use hinted rasterization according rendering mode; vector/export path can use outlines/native PDF text. Color fonts handled explicitly.

# Pixel preview

View mode simulates target raster resolution: snap display to target pixel grid and use nearest/defined pixel sampling after document evaluation. It never mutates vector geometry.

# Retina/DPR

Document zoom independent from devicePixelRatio. Render target physical pixels = logical canvas size × DPR; overlay handle sizes remain logical px.

# Selection overlays

Overlays render in final UI phase and are not color-managed as artwork except accessibility contrast policy. They never enter export.

# Tests

Subpixel paths, thin strokes, mips, rotated rasters, glyph baselines, mixed DPR, pixel preview at noninteger zoom and screenshot goldens.