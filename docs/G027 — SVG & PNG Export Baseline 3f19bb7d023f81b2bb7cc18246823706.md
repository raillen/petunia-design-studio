# G027 — SVG & PNG Export Baseline

# Goal

Complete first usable output path for vector and rasterized artwork.

# Depends

G026, G010–G012, G022.

# Primary

editor-engineer + renderer-engineer.

# Skills

svg-engineering, serialization, rendering-2d, filesystem-security.

# Deliverables

ExporterRegistry; ExportRequest/Report; PNG export from offscreen final renderer with dimensions/alpha/basic color metadata; SVG writer for shapes/transforms/solid/gradient/stroke; temporary output + validation + atomic commit; basic fidelity analysis.

# Acceptance

Vertical slice exports PNG visually matching canvas final-quality fixture and SVG reopens in reference SVG viewer/Petunia importer fixture preserving supported geometry/appearance.

# Tests

Transparent PNG, scale variants, Unicode filename, SVG transforms/gradients/strokes, unsupported feature report, disk failure and output validation.

# Non-goals

PDF, CMYK/spot, advanced effects.