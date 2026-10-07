# 29.3 — RasterLayer, TileSet, PixelMask, Selection & Channel Schemas

# RasterLayer

Common Object + RasterResourceId/TileSetId, pixelExtent, PixelFormatId/descriptor, source ColorSpace/Profile ref, optional original ResourceId, transform and appearance/masks.

# TileSet

Tile size/version, extent/origin, pixel format, canonical mip level 0, sparse tile index/resource mapping and codec capability.

# PixelMask

MaskId/object relationship, TileSet scalar format, transform/alignment to target, invert/enabled/link state.

# PixelSelection

Normally session/edit state: extent, tile mask, feather/derived edge settings. When stored as channel/mask, converted to canonical respective record.

# Channel

Custom alpha/channel resource record with name, scalar tile data and semantic channel kind. Process document channels are derived from raster/color model rather than duplicated canonical resource unless user edits custom channel.

# Bounds

Pixel extent and object transform are distinct; cropping does not silently discard tiles unless Trim command.

# Validation

Pixel format/profile compatibility, tile coordinates/codec limits, resource hash/index, finite transforms and mask target compatibility.