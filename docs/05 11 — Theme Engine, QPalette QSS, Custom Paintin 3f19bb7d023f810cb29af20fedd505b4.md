# 05.11 — Theme Engine, QPalette/QSS, Custom Painting, SVG Icons & Density Scaling

# Theme source

Design tokens JSON is canonical. Theme compiler produces Python/Qt token objects, QPalette roles, limited QSS and icon palette rules.

# QSS policy

Use QSS for broad styling where predictable; avoid giant CSS-like cascade controlling geometry/behavior. Complex professional controls use custom paint/delegate with token access.

# Palette

Semantic surfaces/text/accent/error/warning/selection mapped to QPalette where native widgets benefit.

# Icons

SVG loaded through IconRegistry by IconId. State tinting uses semantic roles. Raster caching by size/DPR/theme bounded.

# Density

Component metrics resolve through DensityProfile. Comfortable/Compact alter heights/padding while retaining minimum interaction hit region.

# Custom painting

Use QPainter for UI chrome/widgets only. Document content renderer remains native renderer.

# Theme switching

Runtime theme switch invalidates UI palette/icons and canvas chrome overlays, not document render caches except display/selection colors that actually depend on view theme.

# Plugin UI

Declarative plugin controls inherit token/theme automatically.