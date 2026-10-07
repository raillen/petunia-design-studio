# 24.1 — Native/Raster Format Capability Matrix: PTND, PNG, JPEG, WebP, TIFF, AVIF/HEIF

# Matrix dimensions

For each format record Import, Place, Export, Roundtrip plus bit depth, alpha, ICC, EXIF/XMP, animation/multipage, lossless/lossy and security codec.

# PTND

Canonical full semantics.

# PNG

Raster import/place/export; 8/16-bit RGB/Gray/alpha where codec supports; ICC; no CMYK/spot; metadata limited policy. Vector/text/effects rasterize on export.

# JPEG

Raster 8-bit typical; no alpha; ICC/EXIF; lossy quality/subsampling. CMYK JPEG import requires explicit codec/color tests; export support only if product chooses and validates.

# WebP

Lossy/lossless, alpha; exact bit depth/features tied codec. Animation not V1 document timeline feature; import first frame or explicit unsupported warning rather than silent ambiguity.

# TIFF

Professional raster target: multi-bit-depth, RGB/CMYK/Gray, alpha, ICC, compression options. Multipage/layers only if explicitly implemented; do not equate TIFF pages with Petunia layer roundtrip.

# AVIF/HEIF

Optional after codec/license/platform/security assessment. Capability matrix must name exact supported bit-depth/HDR/profile metadata.

# Tests

External decoders/encoders, malformed corpus, color charts, alpha, metadata privacy and deterministic option mapping.