# 25.3 — Raster Formats Matrix: PNG, JPEG, TIFF, WebP, AVIF/HEIF

# PNG

Import/export RGBA/Gray baseline; 8/16-bit as library supports; alpha and ICC/gamma metadata. Document effects/layers flatten to final pixels on export.

# JPEG

RGB/Gray baseline; no alpha. Quality/subsampling/progressive settings explicit. ICC and metadata preservation policy. CMYK JPEG import/export only if codec/color testing proves support.

# TIFF

Professional raster target: RGB/CMYK/Gray, 8/16 and float where supported, alpha, ICC, compression choices. Layered TIFF is not assumed unless adapter explicitly implements a profile.

# WebP

Lossy/lossless RGBA/RGB export/import; metadata/profile capabilities tested. No document layers.

# AVIF/HEIF

Post-V1/optional unless codec redistribution/security/platform support passes gates. HDR/wide-gamut semantics require explicit profile/transfer handling.

# Common export

Area/Surface/selection, dimensions/scale, resampling, bit depth, target profile, alpha/background, metadata and dithering.

# Fidelity

All raster exports are destructive projections of vector/text/live state by definition; ExportReport says Rasterized projection rather than pretending roundtrip editability.

# Tests

Profiles, alpha, odd dimensions, huge images, metadata, malformed files, quality settings and external decoders.