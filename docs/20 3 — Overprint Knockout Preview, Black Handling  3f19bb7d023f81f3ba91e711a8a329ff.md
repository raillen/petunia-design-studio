# 20.3 — Overprint/Knockout Preview, Black Handling & Transparency Flattening

# Overprint

Fill and stroke overprint flags are separate properties. Default process behavior knockout unless output semantics/black-overprint option explicitly modifies.

# Black handling

Preset may offer “overprint pure black” policy only with clear scope and standards-compatible output. Rich black is ordinary CMYK color, not magical display color.

# Simulation

Overprint Preview uses separation-domain composition approximation/authoritative path close to PDF output semantics. Ordinary RGB renderer must not fake it with Multiply blend.

# Knockout

Knockout removes underlying ink on affected plates according object coverage. Spot/process interactions tested.

# Transparency flattening

For PDF versions/targets lacking live transparency, Flattener partitions artwork into vector/raster regions under resolution/quality policy. Text/vector preservation prioritized where possible.

# Flattening preflight

Reports affected objects, rasterization DPI and potential spot/overprint change. PDF/X-4 path should avoid flattening where permitted.

# Tests

Black text over colored background, spot overprint, transparent shadow crossing spot artwork and flattened visual comparison.