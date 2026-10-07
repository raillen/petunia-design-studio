# 25.4 — PSD/PSB Compatibility Roadmap & Layer Semantics Matrix

# Status

High-value Post-V1 candidate. No blanket “PSD support” until capability rows are implemented/tested.

# Candidate import rows

Layer hierarchy; raster layers; vector masks; pixel masks; blend modes; opacity; adjustment layers; smart objects; text; layer effects; channels; spot channels; clipping groups; artboards; metadata; color modes.

# Fidelity grades

Each feature has Exact/Equivalent/Approximate/Rasterized/Unsupported. Unsupported Photoshop-specific objects should be preserved opaquely only if format/library architecture safely permits; otherwise reported.

# Text

Font matching, text engine differences and paragraph/text-box reconstruction make Exact editability difficult. Preserve text when mapping verified, otherwise explicit approximation.

# Smart objects

Could map to embedded/linked Petunia document/resource abstraction if semantics align; never fake with ordinary group.

# Effects

Common shadows/glows/blends can map to Petunia EffectNodes if parameters/math verified. Unknown proprietary effects rasterize/unsupported with report.

# Export

Writing PSD is separate, harder capability from import. V1 roadmap can prioritize import before export.

# PSB

Large-document variant is separate profile with size/index constraints.

# Testing

Adobe-generated fixture corpus across versions/features, roundtrip visual diffs, layer-tree semantics and external reopen.