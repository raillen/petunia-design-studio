# 24.4 — PSD/EPS/AI/IDML/DXF-DWG Roadmap Matrices & Honest Compatibility Grades

# Principle

These formats are high-value but complex/proprietary/legacy. Documentation must prevent “supports PSD” from meaning a weak flattened image without disclosure.

# PSD

Define layer types, raster, text, vector masks, adjustment layers, blend modes, smart objects/effects one by one. Initial import could preserve common raster/layer structure; export milestone separate. Unsupported proprietary effects yield report/baked fallback only if source provides composite.

# EPS

Legacy vector/print interchange with PostScript constraints, no modern transparency. Export requires flatten/expand; import security-sensitive interpreter/parser.

# AI

Modern AI often PDF-compatible container but proprietary data. Import can use PDF-compatible representation where available and label result as PDF reconstruction, not AI-native feature support.

# IDML

Post-V1 publishing interchange candidate: pages, frames, text styles, images. Requires dedicated layout mapping and cannot preserve arbitrary InDesign behavior.

# DXF/DWG

CAD vectors/units/layers may map Design geometry but typography/dimensions/blocks need matrix. DWG licensing/library choice separate ADR.

# Status fields

Research / Parser Spike / Import Preview / Import Supported / Export Supported / Certified corpus. Release UI only exposes actual stage.

# Tests

Real licensed/open fixtures, external app roundtrip where legal, feature-by-feature report.