# 22.2 — Brush Preset & Brush Pack Format

# BrushPreset

BrushPresetId, schemaVersion, name/tags/category, tip definition, texture refs, spacing, shape/hardness, scatter, dynamics maps, opacity/flow, rotation, stabilizer defaults, operator compatibility and preview metadata.

# Tip

Procedural circle/shape or image ResourceId. Image tip stores grayscale/alpha interpretation and physical normalization.

# Dynamics

InputAxis -> OutputProperty curve records with control points/interpolation/range/clamp. Unknown axes/outputs versioned capability.

# Pack

BrushPack manifest + resources/textures + presets + license/attribution. ZIP-like safe package profile distinct from plugin package.

# Import

Validate sizes/textures/schema; conflict by PresetId/fingerprint; user chooses replace/duplicate where needed.

# Export

Can package only user-owned resources; external/provider assets require license/provenance check.

# Preview

Generated stroke preview derived and cacheable. Not canonical visual definition.

# Compatibility

Vector brush vs raster brush capability flag. UI filters unsupported tool types.

# Tests

Texture missing, preset migration, dynamics curve, pack traversal attack, duplicate IDs and deterministic preview.