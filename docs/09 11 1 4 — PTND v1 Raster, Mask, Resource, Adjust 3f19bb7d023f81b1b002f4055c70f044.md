# 09.11.1.4 — PTND v1 Raster, Mask, Resource, Adjustment & Data Merge Schemas

# RasterLayer

Fields:

RasterSetId/resource ref, pixel extent, pixel format, source profile ref, object transform, opacity/blend, appearance/effects/masks. Tile bytes live resources/raster.

# RasterSet descriptor

tileSize, width/height, pixelFormat, tileIndex/resource mapping, optional sourceResourceId, canonical level 0 only; derived mips excluded.

# Pixel mask

MaskId/type=pixel, RasterSetId single-channel, transform/extent, invert/feather semantic if live.

# Vector mask

MaskId/type=vector, ObjectId/path reference or owned path payload according ownership model; fill rule; transform.

# Adjustment

AdjustmentKind, version, parameter object, enabled, opacity/blend, masks, scope/nesting represented by layer hierarchy.

# Live filter

EffectKind, params, quality-independent canonical settings, masks and layer/effect ordering.

# Resources

Image original, ICC, font, brush texture, linked file metadata. External resource includes fingerprint/status metadata but OS permission token is not stored in portable document unless secure platform-specific bookmark policy explicitly allows encrypted local metadata outside canonical portability.

# Data source

Provider kind, embedded/linked resource ref, schema fingerprint, field metadata and refresh policy. Sensitive credentials never in document.

# Data binding

BindingId, target ObjectId + PropertyPath, expression AST/string + expressionVersion, formatting/null/fallback and resource fit policy.

# Generated state

Preview record index is session state; generated outputs are ordinary canonical objects/documents, not hidden Data Merge cache.

# Tests

Sparse raster descriptor, missing tile, mask inversion, live adjustment roundtrip, linked data missing, malicious expression/version and resource path traversal rejection.