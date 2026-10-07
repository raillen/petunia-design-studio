# 09.10.3 — ICC Transform Engine, Rendering Intents, Black Point Compensation, Cache & GPU LUTs

# Transform request

Source ColorSpace/Profile, destination, pixel/component format, intent (Perceptual|RelativeColorimetric|AbsoluteColorimetric|Saturation), black-point compensation, proof profile/intent and flags.

# ICC resources

Validate size/header/tag bounds before LittleCMS. Hash profile bytes for ResourceId/cache identity. Built-in defaults ship with clear licensing/source.

# Transform cache

Key complete transform semantics. CPU transform object thread-safe policy documented; per-thread clone if underlying library requires.

# Rendering intents

UI only exposes meaningful intents for conversion/export/proof. Defaults documented. Absolute intent used proof/paper simulation contexts, not accidental default.

# BPC

Black-point compensation option stored in conversion/export/proof request, not global hidden state.

# Proof

Device link/proof chain:

source/working -> proof/output profile -> display profile, with paper/ink simulation flags where supported.

# GPU LUT

Sample authoritative CPU transform into 1D shaper + 3D LUT or suitable representation. LUT resolution chosen by error benchmark. Interpolation tetrahedral/trilinear specified. Out-of-domain/clamping behavior tested.

# Monitor change

Display profile identity change invalidates only display transform/LUT. Canonical/render scene unaffected.

# Tests

ColorChecker/reference patches, intents, BPC, malformed profiles, multi-thread transforms, GPU LUT DeltaE/error thresholds.