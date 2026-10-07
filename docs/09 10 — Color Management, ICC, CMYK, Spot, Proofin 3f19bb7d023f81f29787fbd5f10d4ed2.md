# 09.10 — Color Management, ICC, CMYK, Spot, Proofing & Display Pipeline

# Semantic color

ColorValue includes model/space semantics: RGB, CMYK, Lab, Gray, Spot/Registration where supported. UI display values are projections.

# Profiles

Document working profile/resource IDs; placed resource embedded profile retained. LittleCMS baseline engine; OpenColorIO adapter optional for workflows it fits. Differential/oracle fixtures required.

# Conversion

Transform cache key includes source profile, destination, intent, black-point compensation, pixel format. Conversion never silently changes canonical colors unless command explicitly converts.

# Spot

SpotColor references named ink/swatches with alternate process representation. Export adapters declare support/degradation.

# Display

Render output -> display profile transform -> monitor. Per-window monitor profile can change on move. Soft proof adds proof transform + paper/ink options. Gamut warning overlay is view-only.

# CMYK

Document can use CMYK working context; vector/raster semantics preserved where supported. GPU preview may use LUT/approximation only with validated error bounds/fallback.

# UI

Color panel shows model/profile and numeric units; eyedropper distinguishes composite display sample vs document source values.

# Tests

ICC corpus, malformed profiles, roundtrip tolerances, known patches, monitor switch, soft-proof goldens, spot export/preflight.

[09.10.1 — Canonical ColorValue, ColorSpace, Profiles, Swatches, Spot & Registration Semantics](09%2010%201%20%E2%80%94%20Canonical%20ColorValue,%20ColorSpace,%20Profil%203f19bb7d023f8159b512d19deb621e27.md)

[09.10.2 — Working/Compositing Space Architecture: Linear Light, CMYK Documents & Conversion Boundaries](09%2010%202%20%E2%80%94%20Working%20Compositing%20Space%20Architecture%20L%203f19bb7d023f81cba126d88d65a2b04c.md)

[09.10.3 — ICC Transform Engine, Rendering Intents, Black Point Compensation, Cache & GPU LUTs](09%2010%203%20%E2%80%94%20ICC%20Transform%20Engine,%20Rendering%20Intents,%203f19bb7d023f81d5b317c2800698114f.md)

[09.10.4 — Print Semantics: Overprint, Knockout, Separations, Ink Coverage, TAC & Rich Black](09%2010%204%20%E2%80%94%20Print%20Semantics%20Overprint,%20Knockout,%20Sep%203f19bb7d023f81018b43f99f5c27db5f.md)

[09.10.5 — Soft Proof, Gamut Warning, Display Calibration, Picker Semantics & Color UI Contract](09%2010%205%20%E2%80%94%20Soft%20Proof,%20Gamut%20Warning,%20Display%20Calib%203f19bb7d023f8185a1cfc9cf18f2e426.md)

[09.10.1 — Canonical ColorValue, Color Spaces, Profiles, Alpha & Swatch Semantics](09%2010%201%20%E2%80%94%20Canonical%20ColorValue,%20Color%20Spaces,%20Prof%203f19bb7d023f813498bce73ab4af00fc.md)

[09.10.2 — Working/Compositing Space Policy, Linearization, Precision & Mixed-Color Content](09%2010%202%20%E2%80%94%20Working%20Compositing%20Space%20Policy,%20Linear%203f19bb7d023f8139b9c2c6a2b5197c5b.md)

[09.10.3 — ICC Transform Pipeline, Rendering Intents, BPC, Cache & Display Profiles](09%2010%203%20%E2%80%94%20ICC%20Transform%20Pipeline,%20Rendering%20Intent%203f19bb7d023f81fcb51be48285189887.md)

[09.10.4 — CMYK, Spot Inks, Registration, Overprint, Knockout & Print Separation Semantics](09%2010%204%20%E2%80%94%20CMYK,%20Spot%20Inks,%20Registration,%20Overprint%203f19bb7d023f81e29738c2f9aa7f43a8.md)

[09.10.5 — Soft Proof, Gamut Warning, Ink/Paper Simulation & Color UI/Preflight Contract](09%2010%205%20%E2%80%94%20Soft%20Proof,%20Gamut%20Warning,%20Ink%20Paper%20Sim%203f19bb7d023f81039415cfc91904a394.md)