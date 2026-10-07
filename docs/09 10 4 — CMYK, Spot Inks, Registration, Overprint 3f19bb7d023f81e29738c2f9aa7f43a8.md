# 09.10.4 — CMYK, Spot Inks, Registration, Overprint, Knockout & Print Separation Semantics

# CMYK canonical

CMYK ColorValue stores C/M/Y/K + profile reference. Display preview converts through color engine; compatible PDF/export can preserve process values.

# Spot ink

SpotInk {

SpotId

name

alternate process ColorValue

solidity/tint metadata if supported

}

Object stores SpotId + tint. Multiple spot inks are distinct even with same alternate color.

# Registration

Special color semantic prints on all separations; prohibited for ordinary rich black usage via preflight warning.

# Separations

Print engine can evaluate process C/M/Y/K plus each SpotId into individual coverage planes for preview/preflight. Separation calculation is not generic screen renderer screenshot.

# Overprint

Fill/stroke can declare overprint. Overprint preview simulates interaction of inks according PDF/print rules and output profile/ink model. Default knockout vs overprint explicit.

# Knockout

Normal objects knock out underlying inks unless overprint enabled/advanced group semantics say otherwise. This print compositing path differs from ordinary RGB source-over.

# Rich black/TAC

Preflight computes total area coverage after relevant conversion for target profile and flags threshold. UI may offer rich-black helpers but does not silently rewrite user color.

# Transparency

Transparent objects/effects may require flattening for older PDF targets; preflight reports loss and preserves PDF/X-4 live transparency when target supports.

# Tests

Spot tint separation, registration, overprint black/vector examples, TAC charts, knockout, transparency flattening and PDF external validator/visual comparison.