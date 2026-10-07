# 02 — Design Persona: Vector, Layout & Variable Data

# Design Persona

Design combina Vector + Layout em um workspace profissional. Trocar para Design não converte raster nem altera o documento; apenas muda tool registry, context bar e panel preset.

# Ferramentas baseline

- Move/Select;
- Node;
- Pen;
- Pencil;
- Vector Brush;
- Corner;
- Knife/Scissors;
- Shape tools paramétricas;
- Shape Builder;
- Boolean live/baked;
- Fill/Gradient;
- Transparency;
- Eyedropper;
- Stroke Width/Profile;
- Artistic Text;
- Frame Text;
- Text on Path;
- Artboard/Surface;
- Crop vector/content;
- Measure;
- Zoom/Hand.

# Transformação

Move, resize, rotate, skew, flip, origin, numeric transforms, alignment, distribution, spacing, transform copies, power duplicate. Drag usa preview derivado; commit gera um único Command/undo transaction.

# Vetor

Paths usam segmentos line/cubic com IDs de nodes estáveis durante edição. Node Tool expõe cusp/smooth/symmetric, handles, join/break, close/open, reverse, delete-preserve-shape e snapping.

# Appearance Stack

Cada objeto pode ter múltiplos fills/strokes/effects ordenados. Fill suporta solid, linear/radial/conical/mesh quando implementado, pattern/image. Stroke suporta width, joins, caps, dash, pressure/profile, inside/center/outside quando representável.

# Booleans

Union, subtract, intersect, xor e divide podem produzir resultado baked. Live Boolean preserva operandos e evaluation node não destrutivo.

# Layout

Surface pode representar artboard/page/export region. Design oferece margins, columns, baseline grid, bleed, linked text frames, wrap, style system e reusable SurfaceTemplate.

# Data Merge

CSV/JSON/tabular providers -> schema -> bindings -> preview record -> preflight -> generated document/surfaces. Expressions são sandboxed, deterministic e não executam Python arbitrário.

# GUI

Left tool rail ativa tools; context bar mostra parâmetros do tool; Properties mostra schema completo; canvas HUD mostra medidas/ângulos; Layers e Appearance refletem seleção; status bar mostra modifiers/hints.

# Interoperabilidade

Vetor pode receber pixel masks/effects; raster objects permanecem raster. Convert to Curves, Expand Stroke e Rasterize são comandos explícitos e irreversíveis apenas via undo.

[02.1 — Design Persona Workspace, Default Panels, Tool Groups & Context Model](02%201%20%E2%80%94%20Design%20Persona%20Workspace,%20Default%20Panels,%20T%203f19bb7d023f81e0b24af5a0814b2c58.md)

[02.2 — Vector Construction: Shapes, Pen, Pencil, Vector Brush, Corners & Knife](02%202%20%E2%80%94%20Vector%20Construction%20Shapes,%20Pen,%20Pencil,%20Ve%203f19bb7d023f8180bd28d78be2a8aa9a.md)

[02.3 — Vector Appearance: Fill, Stroke, Gradients, Multi-Appearance, Blending & Effects](02%203%20%E2%80%94%20Vector%20Appearance%20Fill,%20Stroke,%20Gradients,%20%203f19bb7d023f81a1a080f2a40d32e4a6.md)

[02.4 — Vector Composition: Booleans, Shape Builder, Compound Paths, Symbols & Styles](02%204%20%E2%80%94%20Vector%20Composition%20Booleans,%20Shape%20Builder,%203f19bb7d023f81f5aea6c9cff4491c7a.md)

[02.5 — Typography & Layout: Artistic Text, Frames, Linked Stories, Surfaces & Publishing Features](02%205%20%E2%80%94%20Typography%20&%20Layout%20Artistic%20Text,%20Frames,%20%203f19bb7d023f81058565cf1c15ac9ac5.md)

[02.6 — Precision: Grids, Guides, Snapping, Measurement, Transform & Alignment](02%206%20%E2%80%94%20Precision%20Grids,%20Guides,%20Snapping,%20Measurem%203f19bb7d023f81efaf3debb7d3ba15b8.md)

[02.7 — Design Resources: Swatches, Assets, Symbols, Styles, Brushes & Reusable Libraries](02%207%20%E2%80%94%20Design%20Resources%20Swatches,%20Assets,%20Symbols,%203f19bb7d023f81418fe3e130e449a7be.md)

[02.8 — Design Persona V1 Feature Gate, Commercial Workflows & Post-V1 Roadmap](02%208%20%E2%80%94%20Design%20Persona%20V1%20Feature%20Gate,%20Commercial%20%203f19bb7d023f81d78608d134a5fbd115.md)