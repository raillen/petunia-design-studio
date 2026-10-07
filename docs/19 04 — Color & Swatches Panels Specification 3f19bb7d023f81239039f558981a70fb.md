# 19.04 — Color & Swatches Panels Specification

# Color Panel

PanelId ptnd.panel.color. Shows active color target: Fill, Stroke, GradientStop, text color or foreground/background paint color.

# Models

RGB, CMYK, Lab, Gray and Spot-compatible display depending document/target. Profile/space visible when ambiguity matters.

# Controls

Sliders/numeric fields, wheel/plane optional, opacity, Fill/Stroke target switch, swap/default/none, recent colors.

# Color semantics

Numeric edits write semantic ColorValue/SwatchReference through standard property commands; panel never stores display-transformed RGB as canonical without conversion.

# Swatches Panel

PanelId ptnd.panel.swatches. Libraries/scopes: Document, User, Built-in, External/Plugin.

# Swatch types

Process, Global, Spot, Registration, Gradient and pattern if supported. Global/spot badges non-color-coded.

# Actions

Create, duplicate, rename, edit, delete with dependency handling, convert to global/spot, import/export palette, apply to target.

# Scale

Virtualized/list-grid for thousands of swatches with search/categories/favorites.

# Accessibility

Color values readable textually, slider keyboard controls, swatch names/types announced.

# Tests

Profile changes, spot/global propagation, deletion with references, mixed target, palette import conflict and keyboard use.