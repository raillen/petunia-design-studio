# 08.7 — Design Persona: Complete Interface Specification

# Purpose

Design is the primary vector/layout Persona. It combines professional vector editing, typography, light publishing, artboards/pages/Surfaces, reusable resources and data-variable workflows.

**Authority split:** this page owns the Persona-level inventory and default workspace. Detailed interaction grammar is canonical in [08.23 — Tool Interaction Grammar, Context Toolbar, Modifiers & Canvas HUD Contract](08%2023%20%E2%80%94%20Tool%20Interaction%20Grammar,%20Context%20Toolbar,%203df9bb7d023f8144bba3db77c199a947.md); tool-by-tool Design UX and Affinity mapping in [08.24 — Design Persona Tool-by-Tool UX Contract & Affinity Mapping](08%2024%20%E2%80%94%20Design%20Persona%20Tool-by-Tool%20UX%20Contract%20&%20%203df9bb7d023f81e7a45bdac3b72a71ea.md); panel semantics in [08.25 — Panels, Layers, Appearance, Colour, Stroke, Transform, Assets & Symbols UX Contract](08%2025%20%E2%80%94%20Panels,%20Layers,%20Appearance,%20Colour,%20Stroke%203df9bb7d023f814686f4e75513c11bd0.md); typography/precision in 08.26–08.27; cross-Persona workflow in 08.28; and UI evidence in 08.30/14.7. Functional behavior/scope remains owned by 10.x.

# Default workspace

Left: tool rail + optional Pages/Assets dock.

Center: document tabs, context toolbar, canvas.

Right: Layers + Properties stack, with tabs for Color/Stroke/Align/Typography depending task.

Bottom: optional History/Assets/Data dock plus status bar.

# Tool rail groups

Selection:

- Move/Select;
- Node;
- Surface/Artboard.

Drawing:

- Pen;
- Pencil/Freehand;
- Shape tool group: Rectangle, Ellipse, Polygon, Star, Line, custom shapes;
- Vector Brush — **Post-V1 Candidate** unless promoted by the vector functional roadmap;

Content:

- Text: Artistic Text, Frame Text;
- Place Image;
- Gradient/Fill;
- Transparency;
- Eyedropper.

Editing:

- Knife/Scissors;
- Corner tool;
- Shape Builder / interactive region construction — **Post-V1 Candidate** unless promoted; ordinary Pathfinder/boolean commands follow 10.3 V1 scope.

Navigation:

- Hand;
- Zoom.

# Layers panel

Row anatomy reserves positions for:

- disclosure;
- object icon;
- thumbnail optional;
- name;
- effect/mask/symbol badges;
- lock;
- visibility.

Supports nested groups, clip/mask hierarchy, boolean groups, adjustment/effect objects, symbols and text/image/path types. Drop target differentiates reorder vs child/reparent vs clip/mask creation. Inline rename uses Enter/F2 and selects meaningful text without extension-like metadata.

# Properties panel

Contextual but structurally stable sections:

- Transform;
- Appearance;
- Fill;
- Stroke;
- Opacity/Blend;
- Effects;
- Constraints/Alignment where relevant;
- Object-specific parameters;
- Export metadata.

Each section has reset/revert for changed values and mixed-state handling.

# Transform section

Fields: X, Y, W, H, Rotation, Skew where supported; anchor/origin 3×3 control; aspect-lock; relative/absolute mode; flip horizontal/vertical. Units follow document but individual typed units parse where possible.

# Fill and Color

Fill summary row opens/links Color panel. **V1 Required:** None, Solid and canonical gradient types defined by 10.4. Pattern/image fill is a **Post-V1 Candidate** unless promoted. Color panel supports document model selector RGB/CMYK/Lab/Gray as allowed by color engine. Swatches separate document, global/library and recent.

# Stroke panel

Width, alignment, cap, join, miter, dash pattern, markers, pressure/width profile. A compact visual preview appears where it adds value.

# Pathfinder / Boolean

Operations and their V1/Post-V1 status are owned by 10.3; the UI must expose every V1-required Pathfinder operation from that registry and must not use `where supported` as an implementation escape hatch. Unsupported-by-selection states are represented through Action predicates/disabled reasons. Distinguish Live Boolean from destructive Expand/Bake. Tooltips explain effect on editability.

# Align & Distribute

Reference targets and availability are owned by 10.1. The UI discovers them from the semantic alignment contract; it must not hard-code a larger list than the engine supports. V1 must at minimum cover Selection, Key Object and current Surface where semantically valid; additional target modes are exposed only when 10.1 marks them V1/Milestone Required. Commands for horizontal/vertical align and distribute; spacing value field for exact gaps.

# Typography panel

Font family combobox with search/favorites/recent; style; size; leading; tracking; kerning; baseline; OpenType feature access; language; text color; paragraph alignment. Font menu previews must be virtualized and not stall UI.

# Paragraph panel

V1 paragraph controls include alignment/justification, indents, spacing before/after, first-line indent, tabs and TextFrame columns according to 10.6. Keep/widow/orphan controls are **Post-V1 Candidate** because long-document publishing is outside the V1 product charter.

# Text Styles

Paragraph/character style tree/list with create, duplicate, update from selection, redefine, clear overrides. Overrides indicated explicitly.

# Pages/Surfaces panel

Thumbnail + name + dimensions. Supports reorder, duplicate, rename, add, resize, multi-selection, export flags. Layout can switch list/grid. Page numbers are metadata when publishing features enabled.

# Assets/Symbols/Libraries

Thumbnail grid/list toggle; search; categories; drag to canvas; create asset from selection; symbol edit state; detach instance; reveal source. Libraries indicate source/read-only state.

# History

Action list grouped transactionally: a 2-second drag is one Move entry. **V1 uses a standard transactional undo/redo history list.** Arbitrary click-to-preview/time-travel branching is a **Post-V1 Candidate** and may only be enabled after 09.3 defines safe branch semantics; the UI must not invent branching by mutating history directly.

# Layout/publishing-lite controls

Margins/Columns, baseline grid, text frames, text flow links, bleed and page/surface numbering follow 10.6–10.7. Reusable repeated layout content uses the accepted **Symbols + SurfaceTemplate/reference composition** model. Publisher-style Master Pages remain **Out of Scope**.

# Export panel shortcut

Quick export selection/surface presets available from panel; full Export subsystem uses dedicated dialog/window specified elsewhere.

# Design empty states

No document: welcome/new/open UI.

No selection: Properties shows document/surface defaults rather than blank noise.

Unsupported mixed selection: show common properties first and a clear explanation for unavailable object-specific controls.

# Canonical interaction references

This page owns the Design Persona inventory and default workspace. Detailed interaction grammar is canonical in [08.23 — Tool Interaction Grammar, Context Toolbar, Modifiers & Canvas HUD Contract](08%2023%20%E2%80%94%20Tool%20Interaction%20Grammar,%20Context%20Toolbar,%203df9bb7d023f8144bba3db77c199a947.md); tool-by-tool Design UX and Affinity mapping in [08.24 — Design Persona Tool-by-Tool UX Contract & Affinity Mapping](08%2024%20%E2%80%94%20Design%20Persona%20Tool-by-Tool%20UX%20Contract%20&%20%203df9bb7d023f81e7a45bdac3b72a71ea.md); panel semantics in [08.25 — Panels, Layers, Appearance, Colour, Stroke, Transform, Assets & Symbols UX Contract](08%2025%20%E2%80%94%20Panels,%20Layers,%20Appearance,%20Colour,%20Stroke%203df9bb7d023f814686f4e75513c11bd0.md); typography and precision in 08.26–08.27; cross-Persona workflow in 08.28; and UI evidence in 08.30/14.7. Functional behavior and scope remain owned by 10.x.