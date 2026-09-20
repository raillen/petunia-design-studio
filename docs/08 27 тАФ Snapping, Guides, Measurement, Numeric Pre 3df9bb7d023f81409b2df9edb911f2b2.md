# 08.27 — Snapping, Guides, Measurement, Numeric Precision & Spatial Feedback

<aside>
📐

**Precision without mode confusion:** snapping, guides, measurements and numeric entry must help users understand why something moved to a specific position, not merely make objects jump.

</aside>

# Affinity baseline

Affinity provides global snapping, curve snapping, construction snapping, grids, guides, measuring and local Node/Pen snapping controls.[[1]](https://affinity.help/designer2/English.lproj/pages/DesignAids/snapping.html)[[2]](https://affinity.help/designer2/English.lproj/pages/Tools/tools_node.html)

# Snap architecture

Aubrieta distinguishes:

- **global snap policy** — grid, guides, geometry, bounds, centers, key points, pixels;
- **tool-local snap policy** — nodes, handles, construction relationships, width points;
- **temporary modifier override** — suspend or alter snap while held;
- **forced semantic constraints** — e.g. axis/angle constraint.

Local toggles never secretly rewrite global preferences.

# Snap feedback

When a snap occurs show, when useful:

- target marker;
- relation line;
- concise relation label;
- numeric delta;
- source and target semantic IDs in inspection/debug mode.

Multiple candidates are ranked deterministically. Ambiguous candidates may be cycled or inspected rather than producing jitter.

# Snap provenance

The Inspection API can answer: what snapped, to what, under which rule, at what tolerance and what alternatives were rejected. This is essential for agentic UI testing and hard-to-reproduce precision bugs.

# Guides

Guides can be created by rulers, numeric dialog/field or supported direct gestures. Each guide has orientation, position, visibility, lock state and scope. Guide deletion is undoable.

# Grids

Document grid and optional specialized grids are presentation/design-aid state unless the Functional Atlas explicitly persists them in document metadata. Grid appearance uses semantic overlay tokens and must remain visible over light/dark/high-detail artwork.

# Pixel alignment

Force Pixel Alignment is explicit and scoped. It cannot silently alter imported vector geometry without a visible command/result.

# Measurements

Distance and area overlays are transient view state by default, following Affinity's Measure/Area mental model.[[3]](https://affinity.help/designer2/English.lproj/pages/Tools/tools_measure.html)[[4]](https://affinity.help/designer2/English.lproj/pages/Tools/tools_area.html)

# Numeric expressions

Fields accept document units and supported expressions. Parsing errors stay inline and preserve the user's typed value until corrected or canceled. Relative entry has explicit syntax/help and preview.

# Coordinate spaces

All coordinate-bearing fields declare whether they refer to:

- document;
- current Surface;
- parent/local object;
- selection bounds;
- transform origin.

Changing coordinate space never changes geometry until the user edits a value.

# Key object

When alignment/distribution uses Key Object, the key is visually and semantically marked. The user can change it without rebuilding selection.

# Accessibility

Snap events are not announced continuously to screen readers during pointer movement. Keyboard precision workflows expose equivalent numeric controls and concise confirmation for committed operations.

# Performance

Snap candidate search has a frame-time budget and degrades predictably on huge scenes through spatial indexes and candidate limits rather than blocking pointer feedback.

# Gauntlet

Fixtures cover dense geometry, extreme zoom, huge coordinates, rotated objects, mixed Surfaces, local/global toggle combinations, temporary override, high DPI, RTL UI, keyboard-only precision and deterministic candidate tie-breaking.