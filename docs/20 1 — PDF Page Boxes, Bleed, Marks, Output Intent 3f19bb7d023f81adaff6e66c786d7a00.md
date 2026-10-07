# 20.1 — PDF Page Boxes, Bleed, Marks, Output Intent & Print Geometry

# Page boxes

Petunia must model/export MediaBox, CropBox, BleedBox, TrimBox and ArtBox where target PDF/profile supports them. Surface/Page stores semantic trim size; bleed derives from explicit per-edge values.

# Bleed

Bleed is output/document semantic, not visual-only overlay. Objects may extend into bleed. Export can include bleed region with or without marks.

# Printer marks

Crop marks, registration marks, color bars and page information are generated output artifacts outside trim. Their placement, line weight, offset and color/registration semantics are preset parameters.

# Output intent

Professional PDF export can embed output intent profile/reference when target profile requires. Output intent does not silently convert document unless export policy says convert.

# Scale

Print export defaults 100% physical scale unless user explicitly scales. Units/DPI calculations are deterministic and shown in preflight.

# Multi-page

Each Page-role Surface maps to one PDF page in explicit order. Arbitrary Artboards export as selected pages/targets according option.

# Validation

Preflight checks missing bleed where marks/content require it, objects outside intended boxes, invalid page sizes and profile/output-intent mismatches.

# Tests

Asymmetric bleed, mixed page sizes, marks geometry, PDF box inspection, scale and multi-page order.