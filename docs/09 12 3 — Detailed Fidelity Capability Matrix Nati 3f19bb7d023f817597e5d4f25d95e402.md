# 09.12.3 — Detailed Fidelity Capability Matrix: Native, SVG, PDF, Raster & Roadmap Formats

# Matrix dimensions

Every adapter maintains rows for:

geometry, parametric shapes, compound/boolean, strokes/variable stroke, fills/gradients/patterns, text types, fonts/features, pages/Surfaces, raster bit depth, alpha, masks/clips, adjustments/effects, blend modes, ICC, CMYK, Lab, spot/registration, metadata, linked resources, symbols/styles and editability.

# Status vocabulary

Exact — semantic roundtrip/preservation.

EquivalentAppearance — visuals preserved, semantics changed.

Constrained — exact only inside documented limits.

Expanded — live semantic converted to simpler editable primitives.

Rasterized — appearance baked pixels.

Approximate — bounded visual difference.

Unsupported — cannot represent.

# PTND

All released core semantics Exact. Unknown optional extensions preserved opaquely; unsupported required extension gated.

# SVG baseline

Paths/fills/basic gradients/solid strokes: Exact/Constrained depending feature.

Parametric shapes: Expanded to standard SVG shapes/path unless equivalent native form.

Variable width/custom brush: Expanded/Rasterized depending representation.

Artistic text: Constrained by font/CSS/textPath support.

Frame text/linked stories: Approximate/Expanded, no assumption of editable layout parity.

Blend/filter: constrained to SVG supported filters/blends, else rasterize.

CMYK/spot: Unsupported/metadata extension only—preflight.

Multiple Surfaces: one output per Surface.

# PDF baseline

Vector geometry/text/images/gradients/blends/color: high-fidelity target.

Petunia live semantics (symbols, param shapes, editable effects) generally Expanded/EquivalentAppearance because PDF is output/interchange.

Pages map Surfaces/pages per export selection.

Spot/CMYK/ICC/overprint supported as writer matures.

Text editability outside Petunia not guaranteed despite native PDF text objects.

# Raster formats

All vector/text/live semantics rasterized. Color/bit-depth/profile dependent on format.

# PSD roadmap

Layer/raster/text/effect capability matrix must be proven individually; no blanket “PSD support.” Unsupported Petunia vector/layout features may rasterize/expand with explicit report.

# Tests

Matrix row links fixture and external validator/application interoperability evidence.