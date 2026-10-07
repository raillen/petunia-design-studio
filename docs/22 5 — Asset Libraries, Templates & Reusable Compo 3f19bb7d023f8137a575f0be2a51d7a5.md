# 22.5 — Asset Libraries, Templates & Reusable Component Packages

# Asset

Serializable Petunia object subtree + dependency manifest + placement metadata + preview. Asset is not raw screenshot.

# Dependencies

Fonts, images, swatches, styles, symbols, brushes/resources referenced by stable local IDs and remapped on import.

# Placement

Instantiate creates fresh ObjectIds unless asset defines reusable SymbolDefinition import. Relative transforms normalized around asset origin.

# Templates

Document/Surface templates are larger package class with document setup, Surfaces, styles/resources and optional placeholder fields. New Document can instantiate template after capability validation.

# Variants

Asset can expose simple variant metadata/tags, but complex component properties should use Symbols/Styles, not ad-hoc template scripting.

# Packs

Safe archive with manifest, asset payloads, dependencies, thumbnails, license/provenance.

# Tests

Nested resources, duplicate symbols, missing fonts, 1000-asset pack, place multiple times and roundtrip.