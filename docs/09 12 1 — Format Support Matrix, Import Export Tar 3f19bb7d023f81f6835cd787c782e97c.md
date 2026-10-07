# 09.12.1 — Format Support Matrix, Import/Export Targets & Fidelity Roadmap

# Matrix policy

Support is directional and graded: Import, Place, Export, Roundtrip; each format has V1/Post-V1 status, test corpus, security owner and fidelity capability matrix.

# Native

PTND: full canonical edit/save/open.

# V1 raster target

PNG: import/place/export, alpha/profile as supported.

JPEG: import/place/export, no alpha.

WebP: import/place/export.

TIFF: import/place/export with professional bit-depth/profile scope defined by implementation.

BMP/GIF: low-priority compatibility import/export where cheap and licensed; not professional fidelity target.

# V1 vector/document target

SVG: import/place/export, editable vector subset + explicit effects/text degradation.

PDF: import/place/export according parser/writer capability; export prioritized for professional output.

EPS: roadmap adapter if demand justifies.

PSD: high-value roadmap import/export, requires explicit layer/effect compatibility matrix; never claim full Adobe parity casually.

AI: PDF-compatible AI variants may import through PDF semantics when legitimate; native proprietary features not assumed.

IDML: Post-V1 layout interchange candidate.

DWG/DXF: Post-V1 CAD/vector interchange candidate subject licensing/parser security.

# Color

Each row lists RGB/CMYK/Lab/Gray/ICC/spot support independently.

# Text

Editable text, embedded fonts, text-on-path and variable features graded separately.

# Pages/Surfaces

Single/multiple page/artboard capability explicit.

# Testing

Real-world fixtures from allowed/open sources plus generated edge corpus; outputs opened by representative external apps where practical.