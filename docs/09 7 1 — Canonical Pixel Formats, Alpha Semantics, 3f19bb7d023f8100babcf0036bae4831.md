# 09.7.1 — Canonical Pixel Formats, Alpha Semantics, Layout & Working Precision

# Canonical pixel format enum

V1 must explicitly support a closed tested set. Recommended initial families:

- Gray8/Gray16;
- RGBA8_UNORM;
- RGBA16_UNORM;
- RGBA16_FLOAT for high-quality working/intermediate use;
- optional RGBA32_FLOAT for specific filters/export paths;
- CMYK8/CMYK16 only if authoritative color pipeline proves correct semantics.

# Storage layout

Canonical editable raster storage is interleaved channels unless benchmarks demonstrate planar advantage for a dedicated mode. Tile header records exact format; no implicit “native”.

# Alpha

Define straight-vs-premultiplied at every boundary.

Recommended:

- canonical tile data: straight alpha or documented canonical choice;
- compositor intermediates: premultiplied;
- codecs/import/export convert explicitly.

Never blend straight-alpha values with premultiplied formulas.

# Channel range

UNORM 0..max; float nominal 0..1 but may preserve out-of-range intermediate values where adjustment/compositing semantics require. Export clamps/tone-maps according target.

# Color association

Every RasterLayer/Resource has source/document color-space semantic reference. Pixel bytes alone never imply sRGB.

# Endianness

Serialized PTILE defines byte order independent from host. In-memory representation follows explicit codec conversion.

# Alignment/stride

Tile buffer exposes width, height, row stride, channel/element format. SIMD/GPU upload code cannot assume tightly packed rows unless flag says so.

# Conversion

PixelFormatConvert and ColorTransform are separate conceptual operations, combinable in optimized pipelines but independently testable.

# Tests

Alpha edge compositing, 8↔16↔float conversion, endian fixture, out-of-range float, transparent colored pixels and profile association.