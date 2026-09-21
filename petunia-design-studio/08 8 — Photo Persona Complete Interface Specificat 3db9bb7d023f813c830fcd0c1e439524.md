# 08.8 — Photo Persona: Complete Interface Specification

# Purpose

Photo is a raster/image-editing Persona over the **same hybrid document**. Switching to Photo never exports/reimports and never implicitly rasterizes vector objects.

**Authority split:** this page owns the Photo Persona inventory/default workspace. Shared tool interaction grammar is canonical in [08.23 — Tool Interaction Grammar, Context Toolbar, Modifiers & Canvas HUD Contract](08%2023%20%E2%80%94%20Tool%20Interaction%20Grammar,%20Context%20Toolbar,%203df9bb7d023f8144bba3db77c199a947.md); mixed Design↔Photo workflow in [08.28 — Design ↔ Photo Personas, Mixed Workspace Profiles & Cross-Discipline Flow](08%2028%20%E2%80%94%20Design%20%E2%86%94%20Photo%20Personas,%20Mixed%20Workspace%20P%203df9bb7d023f81faaae5e02b4a6eb1b3.md); Photo tool-by-tool UX in [08.31 — Photo Persona Tool-by-Tool UX Contract & Affinity Pixel/Photo Mapping](08%2031%20%E2%80%94%20Photo%20Persona%20Tool-by-Tool%20UX%20Contract%20&%20A%203df9bb7d023f810cada6d30e5b867888.md); adjustment/mask/channel/analysis UX in [08.32 — Photo Layers, Adjustments, Masks, Live Filters, Channels & Analysis UX](08%2032%20%E2%80%94%20Photo%20Layers,%20Adjustments,%20Masks,%20Live%20Fil%203df9bb7d023f81609273c08be65cd5a2.md); and UI evidence in 08.30/14.7. V1/Post-V1 behavior remains owned by 10.9–10.10 and 12.8.

# Default workspace

Left: Photo tool rail + Navigator/Presets/History optional stack.

Center: document tabs + Photo context toolbar + canvas.

Right: Histogram/Color, Adjustments, Layers/Channels, Properties.

Bottom: status bar with pixel dimensions, zoom, active tool, mask/proof/snap states.

# Tool rail

Selection:

- Rectangular/Elliptical Marquee;
- Freehand/Lasso;
- Object/subject semantic selection — **Post-V1 Candidate**; V1 does not depend on ML/AI subject selection;
- Selection Brush;
- Flood selection where appropriate.

Painting:

- Paint Brush;
- Eraser;
- Mixer/Smudge — **Post-V1 Candidate**;
- Clone Stamp — **Post-V1 Candidate** after raster tile/undo foundations are stable;
- Healing/Inpainting — **Post-V1 Candidate**; no AI/inpainting dependency is required for V1.

Tone/content:

- Gradient;
- Fill;
- Eyedropper/Color Sampler;
- Crop/Straighten;
- Dodge/Burn brush tools — **Post-V1 Candidate**;
- destructive Blur/Sharpen brush tools — **Post-V1 Candidate**; nondestructive Sharpen/Blur filters follow the V1 adjustment/filter engine.

Shared:

- Move;
- Text;
- Shape;
- Hand;
- Zoom.

# Brush context toolbar

Required controls:

- preset thumbnail/name;
- size;
- hardness;
- opacity;
- flow;
- spacing/stabilizer when applicable;
- pressure toggles;
- blend mode;
- symmetry/mirror painting — **Post-V1 Candidate**;

A compact brush HUD may open near pointer on shortcut, showing size/hardness and allowing immediate adjustment.

# Brush panel

Sections:

- preset categories;
- search;
- favorite/recent brushes;
- thumbnail grid/list;
- dynamics editor;
- shape/spacing;
- opacity/flow dynamics;
- pressure/tilt mapping;
- texture/scatter dynamics — **Post-V1 Candidate**;

Changes show reset/revert when based on preset. Saving modified brush creates new preset or updates user preset, never silently mutates bundled preset.

# Layers in Photo

Same canonical Layers panel as Design with raster-specific badges/thumbnails. Adjustment and mask rows are first-class objects. Pixel layers show thumbnail, color-space/bit-depth indicator only when diagnostically useful.

# Channels panel

Lists RGB/CMYK/Gray/etc. channels according to document/pixel surface plus Alpha and named masks. Features:

- visibility preview;
- channel selection for editing where supported;
- load channel as selection;
- save selection as channel/mask;
- duplicate/delete custom alpha channels;
- shortcut hints optional.

# Adjustments panel

Grid/list of nondestructive adjustment types:

- Levels;
- Curves;
- Exposure;
- HSL;
- White Balance;
- **Post-V1 Candidate catalog entries unless separately promoted:** Brightness/Contrast, Color Balance, Black & White, Vibrance, Photo Filter and Selective Color.

The panel catalog is generated from registered adjustment descriptors; it must not visually promise an adjustment whose engine contract is not implemented.

Click creates adjustment object/layer and opens Properties. Hover may show short description, never animated preview heavy enough to stall.

# Adjustment Properties

Each adjustment owns a purpose-built editor but follows common anatomy:

- preset selector;
- channel/model selector;
- graph/controls;
- reset;
- before/after preview toggle;
- clipping/mask indicator;
- blend/opacity reachable without changing panel family.

# Histogram

Modes: RGB composite, channel, luminosity; input/output statistics optional. Histogram computes asynchronously and shows stale/loading state rather than blocking paint. Clipping warnings toggle highlights.

# Curves UI

Graph with:

- histogram background;
- diagonal baseline;
- draggable control points;
- channel selector;
- black/gray/white eyedroppers where valid;
- input/output fields;
- reset;
- smooth/freehand curve drawing — **Post-V1 Candidate**; V1 supports point-based Curves editing.

Point selection supports Delete and keyboard nudging.

# Levels UI

Histogram + black/midtone/white input sliders + output levels. Numeric fields mirror handles. Modifier/fine adjustment supported.

# Masks

Mask state is visible in layer row. Mask controls:

- add raster mask;
- add vector mask where supported;
- invert;
- disable/enable;
- unlink/relink transform behavior;
- refine;
- feather/density properties;
- show overlay.

Mask overlay color/opacity configurable for visibility and accessibility.

# Selections

Marching-ants style must remain visible over light/dark content. Optional overlay mode. Floating selection context bar can expose Add/Subtract/Intersect, Feather, Refine, Invert, Clear.

# Crop

Crop overlay dims outside area, displays rule-of-thirds/grid options and straighten control. Context toolbar: aspect ratio preset, W/H, rotation/straighten and Apply/Cancel. **V1 crop is nondestructive by default.** Permanently deleting cropped pixels is a separate explicit destructive command/Post-V1 option unless 10.9 specifies a safe V1 command; do not hide destruction behind a casually toggled checkbox.

# Navigator

Thumbnail with viewport rectangle, zoom field, fit controls. Drag rectangle pans canvas. Navigator refresh throttled and asynchronous.

# Presets

Preset browser supports categories, preview thumbnail, favorite, import/export preset. Applying adjustment preset remains nondestructive.

# Proof/color controls

Proof Colors and soft-proof profile selection are **V1 Required** professional color workflows; Gamut Warning is V1 when supported by the accepted Color Engine contract. The UI must expose loading/unavailable-profile diagnostics rather than hide proofing behind an unspecified “when ready” state. Do not place high-frequency Photo color controls in obscure Preferences only.

# Pixel info/status

Status bar can show pixel dimensions, PPI metadata, color model/profile and bit depth. Info panel can show cursor sample values in current document model plus alternate display model.

# Canonical interaction references

This page owns the Photo Persona inventory and default workspace. Shared tool interaction grammar is canonical in [08.23 — Tool Interaction Grammar, Context Toolbar, Modifiers & Canvas HUD Contract](08%2023%20%E2%80%94%20Tool%20Interaction%20Grammar,%20Context%20Toolbar,%203df9bb7d023f8144bba3db77c199a947.md); mixed Design↔Photo workflow in [08.28 — Design ↔ Photo Personas, Mixed Workspace Profiles & Cross-Discipline Flow](08%2028%20%E2%80%94%20Design%20%E2%86%94%20Photo%20Personas,%20Mixed%20Workspace%20P%203df9bb7d023f81faaae5e02b4a6eb1b3.md); Photo tool-by-tool UX in [08.31 — Photo Persona Tool-by-Tool UX Contract & Affinity Pixel/Photo Mapping](08%2031%20%E2%80%94%20Photo%20Persona%20Tool-by-Tool%20UX%20Contract%20&%20A%203df9bb7d023f810cada6d30e5b867888.md); adjustment/mask/channel/analysis UX in [08.32 — Photo Layers, Adjustments, Masks, Live Filters, Channels & Analysis UX](08%2032%20%E2%80%94%20Photo%20Layers,%20Adjustments,%20Masks,%20Live%20Fil%203df9bb7d023f81609273c08be65cd5a2.md); and UI evidence in 08.30/14.7. V1/Post-V1 behavior remains owned by 10.9–10.10 and 12.8.