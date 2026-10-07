# 22.3 — Swatch, Gradient & Palette Library Format

# Palette

PaletteId, name, scope, colors/swatches ordered/grouped, schemaVersion, provenance.

# Swatch types

Process ColorValue, Global Color, SpotColor, Registration, GradientPreset, optional Pattern reference.

# Global semantics

Inside document, global swatch identity is SwatchId. Library swatch applied to document normally creates/imports a document swatch/resource when linkage semantics require.

# Gradient preset

GradientDefinition without object-specific geometry unless preset type includes normalized default axis. Stop colors may reference document/library swatches only through resolved import mapping.

# Import

ASE/GPL/other palette adapters can map supported colors with report. Unsupported spot/profile metadata preserved where format supports; otherwise warn.

# Export

Target capability matrix for process/spot/global/gradient. Never silently convert spot to RGB without report.

# Color profiles

Palette may reference named/profile-independent values; portable packs can embed permitted ICC resources where needed.

# Tests

Spot/global propagation, gradient linked colors, external import/export, missing profile and migration.