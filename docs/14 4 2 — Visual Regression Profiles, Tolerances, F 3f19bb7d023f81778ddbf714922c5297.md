# 14.4.2 — Visual Regression Profiles, Tolerances, Font Rendering & GPU Variation Policy

# Profiles

OS + renderer backend + DPI + theme + density + locale + deterministic fixture. Goldens are per profile when legitimate platform font/window rendering differs.

# Regions

Full shell for layout regressions plus focused crops for controls/canvas overlays. Document rendering goldens separated from UI chrome to isolate failures.

# Tolerance

Exact pixel for deterministic token/icon/layout where possible. Bounded per-channel/perceptual tolerance for GPU/font antialiasing only after measuring normal variation.

# Fonts

Bundle/pin UI/test fonts where possible. Native system font tests are platform-specific and not compared cross-platform pixel-exact.

# GPU

Different production GPUs should match reference within visual tolerance; renderer-specific bug cannot be waived by enormous fuzzy threshold.

# Update

Golden change requires semantic reason and side-by-side diff review; update tool records Goal/commit.

# Artifacts

CI retains expected/actual/diff and semantic snapshot on failure.