# 08.35 — Qt Pixel Geometry, Color Tokens, Panels, Icons & Microinteraction Contract

# Pixel geometry

Design in device-independent units. Token examples set ranges, not immutable magic numbers. Component gallery verifies at 100/125/150/200% and Retina.

# Separators

1-device-pixel appearance requires DPR-aware painting to avoid blurred half-pixel lines.

# Icons

SVG rendered at target DPR/size with optical alignment. Disabled/selected states derive tokens; do not recolor document artwork icons accidentally.

# Panels

Tab/header/content/footer anatomy consistent. Scrollbars subtle but discoverable; horizontal scroll avoided except genuinely 2D content.

# Hover/press

Hover should not shift layout. Press state uses subtle tonal/position response; tool activation visibly persistent.

# Drag/drop

Insertion line/split target/ghost each distinct semantic visual. Invalid drop uses cursor + visual denial.

# Tool cursor

Custom cursors account for hotspot and HiDPI. Cursor alone never communicates mode; toolbar/status state reinforces.

# Notifications

Transient success low prominence; warnings actionable; errors persist until acknowledged/resolved when needed.