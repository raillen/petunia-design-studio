# 19.12 — Brushes, Masks & Info Panels Specification

# Brushes

PanelId ptnd.panel.brushes. Preset categories, search/favorites, grid/list, source library. Lazy preview strokes and virtualization.

# Brush Settings

Dedicated editor for tip, spacing, dynamics curves, scatter, texture, rotation, smoothing/stabilizer and operator-compatible settings. Modified preset state explicit.

# Masks

PanelId ptnd.panel.masks or integrated Layers view depending workspace. Lists attached pixel/vector masks, enable/invert/link/target and create operations.

# Info

PanelId [ptnd.panel.info](http://ptnd.panel.info). Cursor document coordinate, sampled channel values, profile/space, alpha, selected object metadata and tool-specific measurements.

# Update rates

Info hover updates throttled/coalesced. Brush thumbnails generated background. Mask preview incremental.

# Accessibility

Preset names/categories searchable by keyboard; dynamics graphs have numeric alternatives; current mask target announced.

# Tests

10k presets, missing texture, modified preset save/update, mask target switching and Info sampling under proof mode.