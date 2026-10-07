# 19.12 — Brushes & Brush Settings Panels Specification

# Brushes panel

Preset library grid/list with categories, search, favorites, recent and source library. Selecting preset updates active brush tool defaults.

# Modified state

Changing settings marks current preset Modified; Save New/Update/Revert explicit.

# Brush Settings

Sections Tip, Size/Spacing, Shape, Texture, Scatter, Dynamics, Color/Opacity, Stabilizer and operator-specific. Controls map BrushPreset schema.

# Dynamics editor

Input source dropdown + curve editor + numeric/table alternative; live preview stroke pad can be dedicated test canvas and never modifies document.

# Resources

Missing texture/provider shows diagnostic and preserves preset metadata.

# Performance

Large preset library virtualized; thumbnails generated/cached async.

# Tests

preset modification, dynamics curve, missing texture, import/export and accessibility.