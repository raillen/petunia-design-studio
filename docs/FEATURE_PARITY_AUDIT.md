# Feature Parity & Implementation Coverage Audit (Vector & Photo Personas)

This document provides a comprehensive audit of all tools, live modifiers, filters, adjustments, global settings, and inspector panels across both the **Vector (Design) Persona** and the **Photo (Raster) Persona** in Petunia.

All scope statuses strictly conform to **ADR 12.8 / Canonical Scope Taxonomy**:
- `V1 Required`: Mandatory for product version 1.0.
- `Milestone Required`: Required for current interface/shell integration milestones.
- `Post-V1 Candidate`: Fully specified, deferred until after V1.
- `Out of Scope`: Outside the current product charter.

---

## 1. Interactive Tools — Vector (Design) Persona

The Vector Persona focuses on Bézier curves, parametric geometry, typography, booleans, and non-destructive vector manipulation.

| Tool (`ToolKind`) | Canonical Action (`ActionId`) | Scope | Engine / Shell Status | Freya UI Status | Implementation Gap / Required Work |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Select / Move** | `ptnd.tool.select` | `V1 Required` | ✅ Implemented | ✅ Exposed | Robust hit-testing with spatial index fallback, persistent canvas DOM, and toggle/cycle selection fix. |
| **Node** | `ptnd.tool.node` | `V1 Required` | ✅ Implemented | ✅ Exposed | ✅ Implemented: Context HUD controls for Cusp, Smooth, Symmetric conversion, node deletion, and convert-to-curves. |
| **Point Transform** | `ptnd.tool.point_transform` | `Post-V1 Candidate` | ⚠️ Integrated | ⚠️ Integrated | Folded into standard Transform HUD per ADR 08.33; does not require a standalone rail button. |
| **Pen** | `ptnd.tool.pen` | `V1 Required` | ✅ Implemented | ✅ Exposed | ✅ Implemented: Context HUD modes (Bézier curves, Polygon sharp cusps, Line two-point straight segments), Concluir (finish open path), and convert-to-curves. |
| **Pencil** | `ptnd.tool.pencil` | `V1 Required` | ✅ Implemented | ✅ Exposed | ✅ Implemented: Context HUD 3-level deterministic curve-fitting fidelity (Precise, Balanced, Smooth) and convert-to-curves. |
| **Corner** | `ptnd.tool.corner` | `V1 Required` | ✅ Implemented | ✅ Exposed | Expose per-corner numeric radius controls in inspector and "Bake Corner Geometry" action. |
| **Contour** | `ptnd.tool.contour` | `V1 Required` | ✅ Implemented | ✅ Exposed | ✅ Implemented: On-canvas radial drag handle knob, Euclidean offset distance badge, and dynamic live modifier preview. |
| **Perspective** | `ptnd.tool.perspective` | `V1 Required` | ✅ Implemented | ✅ Exposed | Draw interactive 4-vertex quad handles on canvas overlay. |
| **Knife** | `ptnd.tool.knife` | `V1 Required` | ✅ Implemented | ✅ Exposed | ✅ Implemented: On-canvas dashed red/white cutting guide line overlay with 45° angle constraint via Shift. |
| **Scissors** | `ptnd.tool.scissors` | `V1 Required` | ✅ Implemented | ✅ Exposed | ✅ Implemented: Interactive vertex snapping cursor handle and cut point crosshair on canvas overlay. |
| **Rectangle** | `ptnd.tool.shape.rectangle` | `V1 Required` | ✅ Implementado | ✅ Exposed | Individual 4-corner corner radius editing. |
| **Ellipse** | `ptnd.tool.shape.ellipse` | `V1 Required` | ✅ Implemented | ✅ Exposed | Parametric pie and donut angle controls in context HUD. |
| **Polygon** | `ptnd.tool.shape.polygon` | `V1 Required` | ✅ Implemented | ✅ Exposed | Numeric sides control (3 to 32) in dynamic context toolbar. |
| **Star** | `ptnd.tool.shape.star` | `V1 Required` | ✅ Implemented | ✅ Exposed | Numeric point count and inner-radius ratio in dynamic context toolbar. |
| **Line** | `ptnd.tool.line` | `Post-V1 Candidate` | ❌ Not modeled | ❌ Absent | Simple line drawing handled by Pen in Line Mode for V1. |
| **Shape Builder** | `ptnd.tool.shape_builder` | `V1 Required` | ⚠️ Core Partial | 🟡 In Flyout | Interactive hovered region preview and boolean planar partition synthesis. |
| **Vector Flood Fill** | `ptnd.tool.vector_flood_fill` | `V1 Required` | ⚠️ Core Partial | 🟡 In Flyout | Planar-map boundary detection to fill closed intersecting regions without prior boolean baking. |
| **Artistic Text** | `ptnd.tool.text.artistic` | `V1 Required` | ✅ Implemented | ✅ Exposed | In-place canvas text selection and editing. |
| **Frame Text** | `ptnd.tool.text.frame` | `V1 Required` | ✅ Implemented | 🟡 In Flyout | Bounded paragraph layout with automatic word-wrapping. |
| **Gradient** | `ptnd.tool.gradient` | `V1 Required` | ✅ Implemented | ✅ Exposed | On-canvas interactive gradient vector line [start, end] with color stop handles. |
| **Transparency** | `ptnd.tool.transparency` | `V1 Required` | ✅ Implemented | 🟡 In Flyout | On-canvas opacity gradient vector line with opacity stop handles. |
| **Color Picker** | `ptnd.tool.color_picker` | `V1 Required` | ✅ Implemented | ✅ Exposed | 9x9 pixel magnifying loupe under cursor during canvas sampling. |
| **Style Picker** | `ptnd.tool.style_picker` | `V1 Required` | ⚠️ Core Partial | 🟡 In Flyout | Granular property filtering (stroke-only, fill-only, effects-only, typography-only). |
| **Vector Brush** | `ptnd.tool.vector_brush` | `Post-V1 Candidate` | ❌ Not modeled | ❌ Absent | Skeletal textured stroke application along vector paths. |
| **Artboard / Surface** | `ptnd.tool.artboard` | `V1 Required` | ✅ Implemented | ✅ Exposed | Interactive border handles for resizing and preset selectors (A4, 1080p, Mobile). |
| **Measure** | `ptnd.tool.measure` | `V1 Required` | ✅ Implemented | ✅ Exposed | Transient measurement HUD lines (Euclidean distance, DX, DY, angle). |
| **Zoom & Hand** | `ptnd.tool.zoom` / `pan` | `V1 Required` | ✅ Implemented | ✅ Exposed | Fully operational navigation tools. |

---

## 2. Interactive Tools — Photo (Raster) Persona

The Photo Persona manages raster painting, bitmap selections, masks, and pixel-level edits.

| Tool (`ToolKind`) | Canonical Action (`ActionId`) | Scope | Engine / Shell Status | Freya UI Status | Implementation Gap / Required Work |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Marquee Rect** | `ptnd.tool.photo.marquee_rect` | `V1 Required` | ✅ Implemented | 🟡 In Photo Rail | Render animated marching-ants outline on Vello canvas overlay. |
| **Marquee Ellipse** | `ptnd.tool.photo.marquee_ellipse` | `V1 Required` | ✅ Implemented | 🟡 In Photo Rail | Render animated marching-ants ellipse on canvas overlay. |
| **Lasso** | `ptnd.tool.photo.lasso` | `V1 Required` | ✅ Implemented | 🟡 In Photo Rail | Auto-close on mouse release and raster selection mask polygon calculation. |
| **Selection Brush** | `ptnd.tool.photo.selection_brush` | `V1 Required` | ⛔ Blocked | 🟡 In Photo Rail | Requires raster selection buffer painting with Edge-Snapping algorithms. |
| **Flood Select (Wand)** | `ptnd.tool.photo.flood_select` | `V1 Required` | ⛔ Blocked | 🟡 In Photo Rail | Requires flood fill contiguous color tolerance algorithm connected to raster buffer. |
| **Pixel Paint Brush** | `ptnd.tool.photo.brush` | `V1 Required` | ⛔ Blocked | 🟡 Exposed in HUD | Context HUD controls for Dab Radius (-/+), Hardness, and Opacity. Full destructive drawing awaits tiled pixel layer pipeline (`PixelLayer`). |
| **Pixel Eraser** | `ptnd.tool.photo.eraser` | `V1 Required` | ⛔ Blocked | 🟡 Exposed in HUD | Context HUD controls for Eraser Radius (-/+), Hardness, and Opacity. Full destructive erasing awaits tiled pixel layer pipeline. |
| **Photo Gradient** | `ptnd.tool.photo.gradient` | `V1 Required` | ✅ Implemented | 🟡 In Photo Rail | Direct rendering into raster selection mask or pixel layer buffer. |
| **Crop** | `ptnd.tool.photo.crop` | `V1 Required` | ✅ Implemented | 🟡 In Photo Rail | Aspect ratio constraints (1:1, 16:9, Free) and non-destructive canvas boundary crop. |
| **Clone / Healing / Inpainting** | — | `Post-V1 Candidate` | ❌ Not modeled | ❌ Absent | Advanced photo retouching and texture synthesis (Post-V1). |
| **Dodge / Burn / Smudge / Blur** | — | `Post-V1 Candidate` | ❌ Not modeled | ❌ Absent | Localized tonal and blur brush operations (Post-V1). |

---

## 3. Non-Destructive Live Modifiers (ADR 09.31)

Modifiers alter geometry and rendering without destroying the original parametric shapes and path nodes.

| Modifier (`ModifierKind`) | Scope | Engine (`petunia_design_document`) | Freya UI Status | Implementation Gap / Required Work |
| :--- | :--- | :--- | :--- | :--- |
| **ContourOffset** | `V1 Required` | ✅ Implemented | ⚠️ Partial | Expose join styles (Miter, Round, Bevel) in Properties inspector. |
| **TransparentGradient** | `V1 Required` | ✅ Implemented | ⚠️ Partial | Inspector UI to manage multiple opacity stops along the gradient vector. |
| **Perspective** | `V1 Required` | ✅ Implemented | ✅ Exposed | 4-corner interactive canvas handles with 3x3 perspective mesh and context HUD bake action. |
| **CropRect** | `V1 Required` | ✅ Implemented | ⚠️ Partial | Non-destructive vector bounding box clipping. |
| **Warp / Envelope Mesh** | `Post-V1 Candidate` | ❌ Not modeled | ❌ Absent | Multi-node NxN Bézier mesh warping. |
| **Modifier Stack Inspector** | `V1 Required` | ⚠️ Ready in core | ❌ Absent | UI section to list, toggle, reorder, and **Bake** (commit to curves) modifiers. |

---

## 4. Live Filters & Layer Effects (Layer FX — 10.4 & 10.10)

| Filter / Effect (`EffectKind`) | Family | Scope | Engine | Freya UI Status | Implementation Gap / Required Work |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Drop Shadow** | Layer FX | `V1 Required` | ✅ Implemented | ✅ Exposed | ✅ Implemented: Skia GPU MaskFilter blur with detailed parameter editor for Offset (Dx, Dy), Blur radius, Opacity, and visibility toggle in Dock. |
| **Inner Shadow** | Layer FX | `V1 Required` | ✅ Implemented | ✅ Exposed | ✅ Implemented: Skia clipped inner shadow with MaskFilter blur, Distance, Blur radius, Opacity, and visibility toggle in Dock. |
| **Gaussian Blur** | Live Filter | `V1 Required` | ✅ Implemented | ✅ Exposed | ✅ Implemented: Skia MaskFilter blur with real-time radius adjustments (-1pt, +1pt, +5pt) and visibility toggle in Dock. |
| **Sharpen (Unsharp Mask)** | Live Filter | `V1 Required` | ✅ Implemented | ✅ Exposed | Modeled in core engine with interactive radius and amount controls in inspector dock. |
| **Noise** | Live Filter | `V1 Required` | ✅ Implemented | ✅ Exposed | Modeled in core engine with intensity and monochrome/color toggles in inspector dock. |
| **Outer Glow / Inner Glow** | Layer FX | `Post-V1 Candidate` | ❌ Not modeled | ❌ Absent | 360-degree radial glow dispersion. |
| **Bevel & Emboss** | Layer FX | `Post-V1 Candidate` | ❌ Not modeled | ❌ Absent | Simulated 3D edge lighting. |
| **Liquify** | Photo Persona | `Out of Scope` | ❌ Not modeled | ❌ Absent | Real-time fluid displacement shader engine. |

---

## 5. Tonal Adjustments & Image Analysis (Spec 10.10)

| Adjustment / Analysis Feature | Scope | Engine | Freya UI Status | Implementation Gap / Required Work |
| :--- | :--- | :--- | :--- | :--- |
| **Levels** | `V1 Required` | ✅ Implemented | ✅ Exposed | Non-destructive transfer function, gamma midpoint, and black/white clipping with inspector controls. |
| **Curves** | `V1 Required` | ✅ Implemented | ✅ Exposed | Monotone Cubic Hermite spline transfer curve with S-Curve, Linear, and High Contrast presets and evaluation. |
| **HSL** | `V1 Required` | ✅ Implemented | ✅ Exposed | Hue rotation (-180° to +180°), saturation, and lightness shifts with real-time preview. |
| **Exposure** | `V1 Required` | ✅ Implemented | ✅ Exposed | EV stop multiplier, black offset, and gamma power exponent controls. |
| **White Balance** | `V1 Required` | ✅ Implemented | ✅ Exposed | Temperature (warm/cool) and tint (green/magenta) chromatic adjustments. |
| **Histogram** | `V1 Required` | ✅ Implemented | ✅ Exposed | Interactive multi-channel 32-bin histogram widget (RGB, Red, Green, Blue, Luminance) with mean, shadows %, midtones %, and highlights % stats in Dock adjustments/properties tab. |
| **Channel View** | `V1 Required` | ✅ Implemented | ✅ Exposed | Non-mutating semantic channel inspection (RGB, Red, Green, Blue, Alpha) with quick toggle on Document Tab Strip and real-time monochrome channel mask rendering. |
| **Soft Proofing** | `V1 Required` | ✅ Implemented | ✅ Exposed | Quick toggle on Document Tab Strip simulating US Web Coated (SWOP) v2 press gamut via `petunia_design_color::proof` with live Skia canvas reproduction. |

---

## 6. Global Settings, Tool HUD & Dock Panels

| Area / Feature | Surface ID | Scope | Engine | Freya UI Status | Implementation Gap / Required Work |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Preferences Window** | `ptnd.window.preferences` | `Post-V1 Candidate` | ✅ Implemented | ✅ Exposed | Tabbed dialog with General (Language EN/pt-BR, Accent Palettes), Toolbar Catalog, Performance (GPU/LOD), and Shortcuts. |
| **New Document Dialog** | `ptnd.dialog.new_document` | `V1 Required` | ✅ Implemented | ✅ Exposed | Full presets (1080p, 4K, Square, Mobile, A4, Letter), orientation swap, Bleed (3mm/5mm), Margins (10/20/36pt), and Color Space. |
| **Export Dialog** | `ptnd.dialog.export` | `V1 Required` | ✅ Implemented | ✅ Exposed | Format selection (PNG, SVG, PDF), DPI resolution (72, 144, 300), surface info, transparent background toggle, and export pipeline. |
| **Layers Panel** | `ptnd.panel.layers` | `V1 Required` | ✅ Implemented | ✅ Exposed | Layer reordering with ▲/▼ arrange buttons, inline renaming via ✏️ and RenameObject command. |
| **Properties Inspector** | `ptnd.panel.properties` | `V1 Required` | ✅ Implemented | ✅ Exposed | Integrate dash styles, caps/joins, typography leading/tracking, and FX stack. |
| **Color & Swatches** | `ptnd.panel.color` / `swatches` | `V1 Required` | ✅ Implemented | ✅ Exposed | Color sliders (RGB, HSL, CMYK) alongside existing swatch palettes. |
| **History Panel** | `ptnd.panel.history` | `V1 Required` | ✅ Implemented | ✅ Exposed | Non-destructive branching history timeline navigation. |
| **Navigator Panel** | `ptnd.panel.navigator` | `Post-V1 Candidate` | ✅ Implemented | ✅ Exposed | Draggable viewport rectangle over document thumbnail. |
| **Assets Panel** | `ptnd.panel.assets` | `Post-V1 Candidate` | ❌ Not modeled | ❌ Absent | Reusable component asset library. |
| **Data Merge Panel** | `ptnd.panel.data_merge` | `Post-V1 Candidate` | ✅ Engine in 10.11 | ⛔ Disabled | Variable data CSV/JSON binding UI. |
| **Dynamic Tool Options HUD** | `ptnd.surface.context_toolbar` | `V1 Required` | ✅ Implemented | ✅ Exposed | Dynamic contextual controls for Node (Cusp/Smooth/Symmetric/Delete), Pen (Bézier/Polygon/Line/Finish), Pencil (Precise/Balanced/Smooth), Photo Brush/Eraser (Radius/Hardness/Opacity), Star points, Polygon sides, Contour, Perspective, Measure, Gradient, Text, Selection. |
