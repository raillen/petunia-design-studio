# 00.5.3 — Visual Accessibility: Contrast, Scaling, Handles, Motion & Color Independence

# Contrast

UI semantic tokens meet accessible contrast for text/focus/control states. Canvas overlays use dual-tone/outline strategies to remain visible on light/dark artwork.

# Scaling

UI supports OS scaling and large interface profile; handle/cursor size can increase independently of document zoom.

# Color independence

Selection/lock/error/snap states combine shape/icon/text with color. Color panel supplies numeric/name values; preflight severity not color-only.

# Motion

Reduce Motion disables ornamental transitions and reduces animated viewport effects while retaining immediate state feedback.

# Flash

No rapid flashing or attention animation. Progress uses steady indicators.

# Themes

Dark, Light and High Contrast profiles. User accent cannot reduce required contrast; fallback token chosen.

# Tests

Automated token contrast + visual review at 100/150/200%, high contrast and representative color-vision simulations as supplemental evidence.