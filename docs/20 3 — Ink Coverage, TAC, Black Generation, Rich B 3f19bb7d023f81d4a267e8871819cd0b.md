# 20.3 — Ink Coverage, TAC, Black Generation, Rich Black & Preflight Rules

# Ink coverage

Preflight/preview calculates per-pixel/object effective CMYK coverage after color conversion/proof path. TAC = C+M+Y+K percentage.

# TAC

Target profile/workflow can define recommended/max TAC. Preflight reports areas above threshold with severity and heatmap overlay.

# Black

Distinguish K-only black, rich black and composite neutral blacks. Text/stroke preservation policies can keep 100K black during export when enabled and valid.

# Black generation

Petunia does not invent device-link/GCR/UCR behavior itself when ICC profile transformation owns it. UI exposes resulting process values and preservation options.

# Small text

Preflight rule can flag small multi-ink black text/reversed text because registration risk is practical print issue.

# Hairlines

Rule can flag strokes below physical threshold and registration/overprint interactions.

# Rules

Preflight rules are versioned IDs with severity defaults and configurable thresholds, not hard-coded dialog strings.

# Tests

TAC heatmaps, K-only preservation, rich black, small text, hairline, profile changes and separations consistency.