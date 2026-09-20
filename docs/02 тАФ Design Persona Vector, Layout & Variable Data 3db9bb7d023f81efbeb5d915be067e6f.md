# 02 — Design Persona: Vector, Layout & Variable Data

# Ferramentas Design — baseline

## Seleção e transformação

- Move/Select;
- Node/Direct Selection;
- Transform handles, rotate, scale, skew;
- alignment/distribution;
- snapping engine consistente para grid, guides, nodes, edges, centers, intersections, baselines e spacing.

## Desenho vetorial

- Pen/Bézier;
- Pencil/freehand;
- Rectangle, ellipse, polygon/star e primitive shapes;
- Node add/remove/convert/cusp/smooth;
- Join/Break/Close paths;
- Knife/Scissors;
- Corner tool;
- Width/stroke profile;
- Fill, Stroke, Gradient, Transparency.

## Pathfinder e shape construction

- Union, Intersect, Subtract, XOR, Divide;
- Live Boolean Modifier preservando operands;
- Bake/Expand Boolean explícito;
- Shape Builder / Smart Fill — **Post-V1 Candidate** unless promoted by milestone ADR; when implemented it must reuse arrangement/regions from the canonical Geometry Engine rather than introduce a second region engine;
- Offset Path / Outline Stroke.

## Estrutura e aparência

- Layers/groups;
- clipping/masks;
- Symbols/instances;
- styles;
- ordered EffectChain;
- blend modes;
- live effects;
- ProjectiveTransform, Perspective Grid e Warp/Envelope.

## Tipografia

- Artistic Text;
- Text Frame;
- text-on-path;
- character/paragraph styles;
- OpenType features;
- kerning/tracking/leading;
- columns are **V1 Required** for Text Frames/layout; advanced exclusion/text-wrap shapes are **Post-V1 Candidate** unless explicitly promoted.

## Surface/Layout

- Surface/Artboard tool;
- margins, columns, guides, baseline grid, bleed;
- multi-surface documents;
- reusable layout elements use the accepted **Symbols + SurfaceTemplate/reference-composition** model from V1; Publisher-style Master Pages remain **Out of Scope**.

# Data Merge / Variable Data

Data Merge é capacidade de primeira classe e não um importador CSV acoplado.

`DataSource` V1 starts with CSV/TSV/JSON. SQLite/API/online-sheet adapters are **Post-V1 Candidate** contributions; plugin-provided DataSource adapters are part of the extension architecture when the corresponding SDK contract is available. `DataBinding` maps fields to Text/Image/Object properties; `DataFormatter` handles currency/date/number/string. Conditional visibility is a **Post-V1 Candidate** unless promoted through the functional Data Merge contract.

Fluxos:

- preview por record;
- gerar múltiplas surfaces;
- exportar diretamente sem materializar todas no documento;
- 1 arquivo por record ou PDF multipágina;
- substituição de imagens por campo;
- variable text and image replacement are **V1 Required**; variable colors and conditional visibility are **Post-V1 Candidate** unless promoted by 10.11/ADR.

# Regra de UX

A grande quantidade de features não deve inflar permanentemente a interface. Actions devem estar acessíveis por context toolbar, properties, shortcuts e Command Palette; somente ferramentas de alta frequência ficam sempre visíveis.

# Detailed functional contracts

This page remains the Persona-level scope. Tool-by-tool implementation semantics are canonical in [10 — Functional Engine Atlas](10%20%E2%80%94%20Functional%20Engine%20Atlas%203db9bb7d023f81a2b96dc9446aca0a77.md), especially 10.1–10.8 and 10.11–10.13. New Design features must update both the Persona scope and their detailed functional contract rather than expanding only the UI.