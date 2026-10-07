# 04.8 — Color Stack: LittleCMS, OCIO, ICC Resources, Display Profiles & GPU LUT Strategy

# Primary need

Desktop design requires ICC-native workflows: monitor profiles, placed-image profiles, RGB/CMYK/Lab/Gray conversion, proofing and output profiles.

# LittleCMS

Baseline candidate for authoritative CPU ICC transforms and oracle/reference conversions.

# OpenColorIO

Useful optional layer for scene-linear/VFX-style workflows, LUT pipelines and advanced display transforms; it does not replace ICC document semantics by default.

# GPU

Interactive display conversion may upload sampled 1D/3D LUTs derived from authoritative transform. Approximation must be validated against CPU oracle with bounded error.

# Display profile

Adapter provides per-monitor profile identity. Moving window can rebuild transform asynchronously and atomically swap when ready.

# Caching

Transform key includes profiles, intent, black-point compensation, formats and proof chain. Cache is derived and bounded.

# Spot

Spot is semantic color resource with alternate process color; normal GPU preview uses alternate representation but export/preflight retains spot identity.

# Testing

Reference patches, roundtrip tolerance, malformed ICC, RGB↔CMYK, proof chain and GPU-vs-CPU error corpus.