# 09.8.2 — Compositor Mathematics: Premultiplied Alpha, Blend Modes, Isolation, Pass-Through & Working Space

# Representation

Compositor reference uses premultiplied alpha channels in chosen working color representation. Conversion into working space occurs before blend math; display transform after composition.

# Porter-Duff

Source-over baseline:

Ao = As + Ab(1-As)

co = cs + cb(1-As)

where c is premultiplied. Other compositing operators only if explicitly supported.

# Blend mode

For artistic blend B(Cs,Cb), compute using unassociated color guarded for zero alpha, then combine per W3C/PDF-style reference equation chosen as normative source. Each mode has golden vectors.

# Modes

Normal, Multiply, Screen, Overlay, Darken, Lighten, ColorDodge, ColorBurn, HardLight, SoftLight, Difference, Exclusion, Hue, Saturation, Color, Luminosity. Component modes require defined working color conversion.

# Linear/encoded

Document defines compositing policy. Preferred physically sensible operations may use linear-light RGB, while compatibility modes/export formats can require encoded/PDF semantics. This is an explicit Color/Compositor ADR—not backend dependent.

# Group isolation

Isolated group composites children onto transparent group buffer then group result into backdrop.

Pass-through group allows children interact with parent backdrop subject to group opacity/mask semantics defined exactly.

# Knockout/overprint

Not V1 compositor default; print-specific semantics live Prepress spec and cannot be approximated invisibly.

# Precision

CPU reference uses sufficient float precision. GPU half precision allowed only where error corpus passes thresholds.

# Tests

Per-mode golden pixels, alpha edge cases, nested groups, masks, zero-alpha colors, linear vs encoded fixtures, CPU/GPU differential.