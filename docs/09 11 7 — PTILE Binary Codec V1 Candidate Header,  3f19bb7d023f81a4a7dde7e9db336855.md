# 09.11.7 — PTILE Binary Codec V1 Candidate: Header, Payload, Compression, Checksums & Evolution

# Status

Candidate specification to be finalized by ADR/benchmark before format lock. Once released, codec is immutable except versioned successor.

# Goals

Fast random tile load, bounded decode, exact pixel preservation, simple independent validation, compression, forward codec evolution.

# Proposed header fields

magic "PTIL"

codecVersion u16

headerSize u16

flags u32

pixelFormatId u16

compressionId u16

width u16

height u16

rowStrideOrTightFlag

uncompressedBytes u64

compressedBytes u64

contentChecksum[32] or truncated policy

optional color/profile reference stored in parent index, not repeated per tile.

# Encoding

Canonical little-endian header. Payload raw scanlines in canonical channel layout before compression. Compression None or Zstd V1 candidates.

# Bounds

Decoder validates dimensions, bytes-per-pixel multiplication, stride and compressed/uncompressed limits before allocation/decompression.

# Checksum

Hash/checksum covers uncompressed semantic pixel payload plus format metadata necessary to prevent substitution ambiguity. Manifest separately hashes stored resource bytes if desired.

# Tile transparency shortcut

All-zero transparent tile can be absent; no special zero payload required.

# Evolution

codecVersion dispatches decoder. New compression method can be new compressionId if semantic pixels identical. New pixel format gets explicit PixelFormatId. Unsupported canonical codec is required capability.

# Fuzz

Header truncation, integer overflow, bogus sizes, zstd bombs, checksum mismatch, unknown IDs.

# Benchmark gate

Compare PTILE+zstd vs PNG/TIFF/raw for edit workloads: load latency, random tile access, size, CPU, parallel decode.