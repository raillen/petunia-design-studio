# 09.7.1 — Canonical Pixel Formats, Alpha, Channel Layout, Bit Depth & Color Semantics

# Purpose

Freeze the semantic pixel model independently from codecs/GPU formats.

# PixelFormat descriptor

Fields:

model (RGB|CMYK|Gray|Lab where raster support exists), channel_type (UNorm8|UNorm16|Float16|Float32), alpha (None|Straight|Premultiplied semantic input), channel_order, profile/ColorSpaceId, endian/storage version.

# Recommended V1 canonical working set

RGBA8_UNorm, RGBA16_UNorm, RGBA16F/32F for high precision operations, Gray8/16, CMYK8/16 when document workflows require canonical CMYK raster preservation. Exact required set is locked by ADR before TileStore implementation.

# Alpha

Canonical compositing representation uses premultiplied alpha internally for blend/filter math unless an algorithm explicitly requires unassociated channels. Imports convert straight→premultiplied after color interpretation; exports unpremultiply safely when target requires straight alpha.

# Zero alpha

RGB values under alpha=0 have defined handling to avoid division explosions and fringe artifacts. Preserve hidden color only when semantic requirement/format warrants it; compositor reference path defines expected result.

# Color

Pixel bytes are meaningless without ColorSpace/Profile semantics. TileStore never assumes sRGB globally. Conversion to render working space is explicit derived operation.

# Channel access

Channel editing derives typed channel views; channel order is not exposed as magic byte offsets outside raster module.

# Conversion

PixelConverter handles bit depth, alpha association and color transform as distinct ordered steps with high-precision intermediates. Dithering policy explicit for depth reduction.

# Tests

Roundtrip straight/premultiplied, alpha=0/near-zero, 8↔16↔float, RGB/Gray/CMYK fixtures, endian serialization and SIMD/reference parity.