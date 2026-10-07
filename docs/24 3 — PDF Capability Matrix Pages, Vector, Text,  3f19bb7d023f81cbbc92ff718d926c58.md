# 24.3 — PDF Capability Matrix: Pages, Vector, Text, Transparency, Color & Print

# Export priority

PDF is primary professional interchange/print target.

# Geometry/paint

Paths, fills, strokes, gradients mapped natively when PDF supports. Variable/profile strokes can expand.

# Text

Preserve text with embedded/subset fonts when layout can be expressed exactly. Complex text can still emit positioned glyphs retaining selectable text where encoding mapping supports; otherwise outline with explicit degradation.

# Raster

Images embedded at effective resolution/profile; avoid unnecessary recompression if source-compatible and no edits require re-render.

# Transparency

Modern PDF preserves alpha/groups/blend. Older/PDF-X targets may flatten.

# Color

RGB/CMYK/Gray, ICCBased, Lab and Separation/DeviceN spots according writer capability. OutputIntent for print target.

# Pages

Surface Page sequence maps naturally. Artboard export order explicit.

# Import

PDF import is reconstruction, not guaranteed original editability. Paths/text/images can map canonical; transparency groups/effects approximated according parser capability. Always report fidelity.

# Roundtrip

Never claim PDF as native roundtrip. Export then import may preserve appearance but lose Petunia structure.

# Tests

External PDF viewers, Ghostscript/poppler where appropriate, veraPDF/print validator target, spot/separation samples and text extraction.