# 10.12 — Design ↔ Photo Interoperability, Conversion Commands & Shared Composition Rules

# One document

Persona switch changes UI only. Vector, text, raster, adjustments and masks coexist in one paint hierarchy.

# Shared composition

Opacity, blend, clips, masks, transforms and effects operate through common compositing contracts. Object-specific capabilities remain explicit.

# Vector mask on raster

Vector path/group can serve as live mask without rasterizing source geometry. Renderer evaluates vector mask at output resolution.

# Pixel mask on vector/text

Pixel mask can mask any drawable/group; mask tile resolution/transform defined in document coordinates.

# Raster effects on vector

Live blur/shadow/etc can render vector into intermediate surface without changing canonical vector. Export preflight handles targets that cannot preserve live effects.

# Vector operations on raster

Boolean/path editing do not accept raster by default. User must create selection/vectorize/clip or explicit Rasterize/Trace workflows. UI disabled reason explains.

# Rasterize

RasterizeObjectCommand takes target object(s), resolution/DPI, color context, bounds and effect policy -> new RasterLayer replacing or duplicating. Explicit destructive conversion, preflight summary where text/editability lost.

# Expand

Expand Appearance/Stroke/Live Boolean/Shape converts parametric/live semantics to vector paths while preserving vector editability as far as possible.

# Convert to Curves

Text -> vector glyph paths. Font/style metadata may be retained in command provenance but result is not editable text.

# Convert selection to mask/path

PixelSelection -> vector tracing is algorithmic separate feature, not lossless conversion. Vector shape -> pixel selection rasterizes at selection resolution.

# Embedded document/object

Future placed .PTND can render as linked embedded document snapshot with edit source/open embedded workflow; not V1 unless prioritized.

# Color

All objects render through document color engine. Raster resources retain source profiles until conversion policy. Mixing CMYK/RGB objects does not force silent permanent conversion.

# Commands

Rasterize, ExpandAppearance, ExpandStroke, BakeLiveBoolean, ConvertTextToCurves, VectorMaskFromObject, PixelMaskFromSelection, SelectionFromVector, TraceBitmap candidate.

# GUI

Persona switch preserves selection. Context/Properties shows actions based capabilities. Conversion dialogs only when parameters/fidelity need input; otherwise command + undo.

# Tests

Mixed stack rendering, masks crossing domains, live filter vector, rasterize at multiple DPI, export equivalence, undo and Persona switch no mutation.