# 20.7 — Print Image Resolution, Effective PPI, Scaling, Compression & Font Preflight

# Effective PPI

For placed raster, effective PPI derives source pixel dimensions and final transformed physical dimensions. Nonuniform transforms report X/Y or conservative minimum.

# Rules

Preset defines warning/error thresholds for color/grayscale/1-bit image classes. Upsampling in export does not magically resolve low-source resolution; preflight uses effective source detail.

# Compression

PDF/image export chooses lossless/lossy compression settings with resample threshold. Report any downsampling.

# Linked resources

Missing/changed images error/warn. Embedded low-resolution preview cannot substitute missing high-res source unless explicitly accepted.

# Fonts

Preflight checks missing fonts, unresolved glyphs, embedding restrictions, faux styles, variable font/export support and outlined text policy.

# Text

Overset frame, text outside trim/bleed as configured, tiny text/stroke warnings optional printer preset.

# Tests

Rotated/nonuniform image, low PPI, linked high-res missing, restricted font, unresolved glyph and downsample report.