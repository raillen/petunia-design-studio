# 08.12 — Export, Quick Export, Slices, Batch Export & Preflight UX

# Entry points

File > Export, Quick Export, Export panel/workspace, slice context, batch export, MCP/plugin helpers.

# Export window

Preset/format, target preview/summary, Dimensions, Area, Rasterization, Color, Metadata, Advanced, preflight summary.

# Areas

Whole document, current Surface, selected Surfaces, selection, custom slice/region.

# Formats

PNG/JPEG/WebP/TIFF and SVG/PDF baseline. AVIF/HEIF depend on codec/license. PSD/AI/EPS/IDML/DWG são adapter roadmap com fidelity status explícito.

# Fidelity

Cada degradation mostra object, source feature, strategy e grade Exact/EquivalentAppearance/Approximate/Destructive/Unsupported.

# Color

Target profile + embed/strip + CMYK/RGB/Gray where supported. Soft proof não é baked sem explicit choice.

# Slices

Canvas bounds/labels + panel rename/preset/scales/suffix/folder.

# Batch

Independent rows, shared cancellation, conflict policy Ask/Overwrite/Skip/Rename.

# Quick Export

Usa preset e destination grant aprovados; não esconde destructive conversion.

# Validation

Writer pode reabrir/validate temp output antes de atomic destination commit.