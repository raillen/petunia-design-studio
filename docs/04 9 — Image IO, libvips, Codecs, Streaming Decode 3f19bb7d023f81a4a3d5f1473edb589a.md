# 04.9 — Image IO, libvips, Codecs, Streaming Decode, Metadata & Security Boundaries

# Import architecture

Image adapter inspects bounded header, identifies codec, validates dimensions/bit-depth/profile/metadata limits, then decodes through controlled pipeline.

# libvips

Candidate for efficient large-image and streaming operations. Petunia wraps it; VipsImage does not become document model.

# Decoding

Prefer lazy/tiled or scanline decode when source/library supports it. Huge dimension multiplication checked before allocation.

# Metadata

EXIF/XMP/IPTC handling is explicit. Orientation normalized into semantic transform or import operation. Privacy-aware export can strip metadata.

# Embedded profiles

Extract into ResourceId and preserve source profile metadata. Unsupported/broken profile generates diagnostic and defined fallback, never unchecked pointer/data.

# Codecs

Each codec has maximum dimensions, memory/time budget and fuzz corpus. High-risk parser can move out-of-process later without changing ImportAdapter contract.

# Linked source

Placed linked image can retain original encoded bytes/path reference and build derived decode cache. Editing pixel layer creates Petunia-owned tile storage.

# Export

Encoder writes from immutable pixel stream + profile/metadata options and validates file structure where feasible.