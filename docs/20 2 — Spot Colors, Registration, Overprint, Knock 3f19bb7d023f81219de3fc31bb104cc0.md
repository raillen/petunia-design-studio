# 20.2 — Spot Colors, Registration, Overprint, Knockout & Separations

# Spot inks

SpotColor is a named ink resource with alternate process color and optional vendor metadata. Using spot swatch preserves ink identity through vector/text objects and supported exports.

# Registration color

Special color meaning 100% on all separations. It is reserved for marks and explicit expert use; UI distinguishes from rich black/process black.

# Overprint

Fill and Stroke can carry overprint flags where semantic. Default is knockout according output model unless profile/workflow says otherwise.

# Knockout

Group/object compositing semantics must distinguish visual transparency blending from print knockout/overprint. Screen preview uses separations-aware simulation.

# Separations

Separations Preview lists process plates and spot plates. User can toggle plates, inspect ink contribution and navigate objects producing selected ink.

# Preview

Overprint Preview must use print-color/separation model rather than ordinary RGB blend approximation where fidelity matters.

# Export

PDF adapter maps spot colors and overprint operators when target supports. Unsupported target gets degradation warning and process conversion option.

# Tests

Spot-only object, mixed spot/process gradients restrictions, overprint black, registration marks, plate toggles and PDF inspection.