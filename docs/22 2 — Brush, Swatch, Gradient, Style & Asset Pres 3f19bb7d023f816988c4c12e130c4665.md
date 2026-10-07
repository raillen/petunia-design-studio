# 22.2 — Brush, Swatch, Gradient, Style & Asset Preset Schemas

# BrushPreset

Tip resource/shape, spacing, hardness/falloff, scatter, texture, dynamics mappings/curves, blending/operator, stabilizer defaults and compatible tool kinds.

# Swatch

Process ColorValue, Global template, SpotInk, Registration; palette grouping/order metadata.

# Gradient

GradientDefinition with stops/colors/opacity/spread and optional document-independent color-space policy.

# ObjectStyle

Property map limited to styleable stable PropertyIds, inheritance parent optional, preview metadata.

# TextStyle

Separate Character and Paragraph style schemas; avoid arbitrary widget state.

# Asset

Serialized canonical object fragment + required resource bundle + insertion metadata/preview. Asset import validates fragment with same document schema/capabilities.

# Security

Preset/library data cannot execute arbitrary Python. Plugin-provided executable behavior remains plugin capability, not serialized preset.

# Tests

Import/export each kind, missing texture/font, style inheritance cycle and asset resource collection.