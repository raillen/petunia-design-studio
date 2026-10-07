# 18.30 — Flood Fill Tool Specification

# Target

RasterLayer or PixelMask; respects active PixelSelection.

# Parameters

fill source solid/pattern where supported, tolerance, contiguous, anti-alias, sample Current/Current & Below/All, blend/opacity.

# Sampling

Capture immutable source snapshot at click revision. Color-distance metric and tolerance domain are specified per color model/working space.

# Region

Contiguous flood uses bounded queue/span algorithm with tile awareness. Non-contiguous mode selects/fills all matching pixels and may background for large document.

# Commit

Compute affected mask/tiles off-thread if expensive, then atomic ApplyFloodFill transaction.

# UI

Click point marker optional, progress/cancel for huge fill. Disabled if target format/operator unsupported.

# Tests

tile-spanning region, transparent pixels, selection boundary, sample merged, 16-bit/float, cancellation and undo.