# 20.2 — Separations Preview, Ink Coverage, TAC, Registration & Spot Plates

# Separation model

PrepressAnalyzer evaluates each printable object into coverage for process C/M/Y/K and each SpotId. Registration contributes to all plates.

# Preview

Panel lists plates with toggle/solo, ink name/color and usage count. Canvas re-renders separation composite/solo without document mutation.

# Ink coverage

For target process conversion, compute per-pixel/region total process coverage C+M+Y+K. Heatmap and max/percentile statistics available.

# TAC

PrintPreset has target TAC threshold (e.g. printer/profile recommendation). Preflight flags regions exceeding threshold with severity configurable; it does not automatically alter colors.

# Spot

Spot tint preserved. Unsupported transparency/effect interactions with spot identified as potential rasterization/process conversion.

# Unused plates

Preflight can report unused declared spot swatches optionally; export includes only used plates unless preset says otherwise.

# Tests

Pure CMYK patches, 300%/400% TAC, spot tint gradients, registration marks and separation toggle goldens.