# 09.11.2 — Raster Tile Resource Codec, Compression, Deduplication & Large-Document Storage

# Tile resource

RasterLayer references tile set metadata: extent, tile size, pixel format, profile/color context and mip policy. Tile path is storage location, not identity.

# PTILE

Petunia-specific tile chunk format is optional internal resource codec. Header: magic, codecVersion, pixel format, dimensions/stride semantics, uncompressed length, compression codec, checksum. Exact binary layout requires ADR/spec before implementation.

# Compression

Lossless canonical raster by default. zstd candidate for raw tile blocks after benchmark. Original placed JPEG/PNG may be retained as original resource while decoded/editable tiles are generated according layer semantics.

# Sparse

Transparent/unallocated tiles represented implicitly or indexed absent; huge canvases avoid writing empty data.

# Dedup

Content fingerprint may deduplicate immutable resources/tiles only when semantics/lifetime permit. Never merge editable layers solely by identical bytes if future independent mutation would violate copy-on-write rules.

# Streaming

Reader opens manifest/document first and loads visible/needed tiles lazily. Resource limits known before decompress. Export/full analysis can stream all.

# Mips

Derived mip tiles belong cache or explicitly derived section, not canonical truth unless format revision deliberately specifies otherwise.

# Corruption

Bad tile checksum marks affected resource degraded; document can open with missing tile placeholder/salvage report when safe.

# Migration

Resource codec version migration can rewrite tile chunks without changing document schema version.