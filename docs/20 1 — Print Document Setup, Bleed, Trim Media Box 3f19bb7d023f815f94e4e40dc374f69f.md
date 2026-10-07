# 20.1 — Print Document Setup, Bleed, Trim/Media Boxes, Marks & Page Geometry

# Boxes

Surface/page export derives MediaBox, TrimBox, BleedBox and optional CropBox according PDF semantics. Values are computed from canonical Surface + bleed, not ad-hoc export pixels.

# Bleed

Per-side bleed in document units. Artwork may extend outside trim. Preflight flags insufficient bleed only for objects expected to touch edge under configured heuristic/manual rules.

# Marks

Crop marks, registration marks, color bars, page information and bleed marks are export-generated content outside trim where target/preset requests. They do not become canonical artwork.

# Page ordering

Sequential Surface pages map to PDF pages. Arbitrary artboard layouts require explicit export order.

# Scaling

Print export default 100%. Any scale-to-fit is explicit preset and preflight-visible because it changes physical dimensions.

# DPI

Vector/text resolution-independent. Raster effective DPI computed after transforms for preflight.

# Tests

Asymmetric bleed, marks outside MediaBox sizing, multi-page order, rotated Surface and physical size correctness.