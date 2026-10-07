# 09.10.5 — Soft Proof, Gamut Warning, Display Calibration, Picker Semantics & Color UI Contract

# Soft proof

View configuration: target proof profile, intent, BPC, paper/ink simulation, gamut warning. It is view state/preset and does not mutate objects.

# Gamut warning

Compare source/working colors against proof gamut using ColorEngine algorithm; overlay configurable color/pattern. Warning is approximate diagnostic with defined method, not proof of print impossibility.

# Display

Each top-level window resolves monitor/profile. If OS profile unavailable use explicit fallback and diagnostics. HDR/wide-gamut display support is separate ADR/capability.

# Picker modes

Document Source — inspect canonical object/pixel values if directly addressable.

Composite Document — sample evaluated render before display transform in working/document representation.

Display — optional sampled final displayed color, labeled clearly.

# Average sample

Radius in screen/document pixels defined; color average occurs in linear/appropriate working space, not naive encoded bytes.

# UI values

Color panel can switch model representation. Editing RGB representation of CMYK-linked color may either convert/set canonical model explicitly—never silently reinterpret.

# Delta/value precision

Numeric fields show useful precision but canonical retains higher precision. Copy color supports CSS/hex only when representable and labeled conversion.

# Accessibility

Color swatches expose numeric/name/spot semantics; contrast of UI controls independent from artwork.

# Tests

Monitor switches, proof toggles, sample modes, wide gamut profiles, gamut overlay and numeric roundtrip.