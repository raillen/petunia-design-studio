# 20.3 — Separations Preview, Spot Plates, Registration & Plate Visibility

# Plate model

Process plates C, M, Y, K for CMYK output context plus one plate per SpotColorId. Registration contributes 100% to all relevant plates.

# Separation computation

Evaluate canonical artwork/effects into print-output semantics and derive per-plate coverage. RGB/Lab content converted through selected output profile/intent for preview.

# Panel

List plates with swatch/name/type, visibility toggle/solo, ink percentage under cursor, object/source info optional. Process/spot clearly differentiated.

# View

Turning plate off simulates absence in proof preview only; document unchanged. Solo shows grayscale/ink-color visualization mode.

# Spot alternate

Screen color may use alternate process color, but separation uses spot identity/coverage.

# Gradients/transparency

Spot gradients/transparency supported only if output semantics/writer can preserve; otherwise preflight warns/rasterizes according PDF target.

# Accessibility

Plate list/state textual; cursor ink percentages in Info panel.

# Tests

Process-only, one/multiple spots, registration marks, tint values, transparency and PDF separation comparison.