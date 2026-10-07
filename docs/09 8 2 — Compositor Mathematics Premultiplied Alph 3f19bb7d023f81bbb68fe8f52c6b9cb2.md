# 09.8.2 — Compositor Mathematics: Premultiplied Alpha, Blend Modes, Groups & Isolation

# Canonical compositor rule

All compositing math is specified in one normative contract and implemented by CPU reference first. GPU shaders must match within tolerance.

# Alpha

Compositor works in premultiplied alpha intermediate representation. Source straight-alpha resources are converted before blending. Zero-alpha color channels are normalized according format conversion policy to avoid undefined halos.

# Source-over

Normative Porter-Duff source-over:

Ao = As + Ad(1-As)

Co_premul = Cs_premul + Cd_premul(1-As)

# Blend modes

For separable/nonseparable blend modes, use W3C/PDF-compatible equations or explicitly selected standard reference. Document exact formulas for:

Normal, Multiply, Screen, Overlay, Darken, Lighten, Color Dodge, Color Burn, Hard Light, Soft Light, Difference, Exclusion and HSL component modes.

# Blend color space

Define whether blend function operates in linear-light working RGB or encoded/perceptual space per document/render policy. This cannot be backend-specific.

# Group semantics

**Isolated group:** children composite into transparent offscreen, group opacity/blend applied once to backdrop.

**Pass-through group:** normal group can allow children to blend with external backdrop according defined semantics, but group-level opacity/effects may force isolation.

Rules are explicit and testable.

# Object opacity

Object-level opacity applies after internal multi-appearance composition unless appearance entry specifies its own opacity.

# Knockout

Deferred unless Prepress/advanced blend milestone enables it; no accidental approximation.

# Masks

Mask scalar multiplies source premultiplied color and alpha at defined stage. Vector/pixel mask chain combination order normative.

# CPU oracle

Reference implementation uses high precision float and deterministic formulas. GPU results compared on corpus including transparent edges and nested groups.

# Tests

Every blend mode over known source/destination patches at alpha 0/0.25/0.5/1, nested opacity, pass-through, masks and linear-vs-encoded fixtures.