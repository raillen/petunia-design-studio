# 22.2 — Brush Preset & Gradient/Swatch Library Contracts

# BrushPreset

PresetId, name/tags/category, brush-engine version, tip descriptor/resource, spacing, hardness/falloff, texture, scatter, rotation, base size/opacity/flow, dynamics mappings, stabilizer defaults, blend/operator compatibility and deterministic random settings.

# Dynamics

Input source Pressure/Tilt/Rotation/Velocity/Random -> target Size/Opacity/Flow/Rotation/Scatter/etc through CurveMapping. Unknown target is optional capability.

# Modified state

Selecting preset loads immutable preset values into ToolSettings. User edits become Modified copy; Save New or Update preset is explicit.

# GradientPreset

Type-neutral stop list/color interpolation metadata; applying preset copies or links only if linked-preset feature explicitly exists.

# SwatchLibrary

Process/global/spot/registration/gradient entries. Document global/spot references are semantic resources; ordinary palette color application may copy value.

# Import/export

Palette/brush external adapters map fidelity and unsupported dynamics. Never silently discard spot/global identity when importing into document if target supports it.

# Tests

Missing texture, dynamics migration, deterministic seed, library read-only, swatch reference propagation and conflict resolution.