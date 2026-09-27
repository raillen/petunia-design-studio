# 08.24 — Design Persona Tool-by-Tool UX Contract & Affinity Mapping

# Amendment 2026-09-21 — Petunia identity / Slint binding

All tool UX in this atlas now targets Petunia Design Studio. Affinity remains prior art for ergonomic workflows only. Every V1 tool receives a stable ToolId, ActionId entry paths, Petunia semantic IconId/TextId, Slint presentation binding, interaction-state test, Undo/Redo evidence and save/reopen evidence where document state changes.

<aside>
✒️

**Tool-by-tool UX map.** This complements 10.x engine semantics. It specifies how major Design tools should feel and how Affinity prior art is adopted or improved.

</aside>

# Tool definition template

Purpose → Entry → Selection prerequisites → Canvas behavior → Context controls → Modifiers → Snap → Preview/commit → Undo → Error/disabled → Accessibility → Automation → Affinity divergence → Fixtures.

# Move / Select

Affinity Move selects, moves, rotates and resizes, with auto-select modes, transform origin, alignment handles and transform-separately behavior.[[1]](https://affinity.help/designer2/English.lproj/pages/Tools/tools_move.html)

Petunia improvements: visible Auto Select state; predictable selection cycling; explicit key-object marker; transform-origin scope; selection-box hiding as workspace preference only.

# Node

Affinity Node converts node types, breaks/closes/joins/reverses curves and exposes local/construction snapping.[[2]](https://affinity.help/designer2/English.lproj/pages/Tools/tools_node.html)

Petunia adds node-role shape encoding, accessible labels, optional orientation display independent of colour, compact snap explanations and topology preview where operations alter node/segment count.

# Point Transform

Affinity rotates/scales around a movable origin placed on own or other geometry and reports delta, scale and angle.[[3]](https://affinity.help/designer2/English.lproj/pages/Tools/tools_pointTransform.html)

Petunia integrates this with the same Transform HUD and Property schema used elsewhere rather than creating isolated precision rules.

# Pen

Reference modes include Pen, Smart, Polygon, Line, Preserve Selection, Add New Curve and Rubber Band preview.[[4]](https://affinity.help/designer2/English.lproj/pages/Tools/tools_pen.html)

Petunia keeps the useful mode family, makes append-to-current-path state persistent and obvious, and provides a visible close-path target near the first node.

# Pencil / Freehand

Affinity provides Sculpt, Auto Close, Smoothness, Pressure/Velocity controller and Rope/Window stabilizers.[[5]](https://affinity.help/designer2/English.lproj/pages/Tools/tools_pencil.html)

Petunia adds live smoothing preview, stabilizer latency budgets and explicit distinction between raw sampled input and fitted editable path in diagnostics.

# Stroke Width

Affinity edits pressure points directly on a curve and can snap widths.[[6]](https://affinity.help/designer2/English.lproj/pages/Tools/tools_lineWidth.html)

Petunia uses semantic width handles, numeric entry and visible interpolation. Width remains non-destructive appearance until Expand Stroke.

# Corner

Affinity supports rounded, chamfer, concave and cutout corner types, radius and Bake Appearance.[[7]](https://affinity.help/designer2/English.lproj/pages/Tools/tools_corner.html)

Petunia names irreversible conversion explicitly as **Bake Corner Geometry**.

# Contour

Affinity offsets outlines inward/outward with join/cap/fill rules and Bake Appearance.[[8]](https://affinity.help/designer2/English.lproj/pages/Tools/tools_contour.html)

Petunia adds signed radius semantics, visible zero crossing and degenerate/overflow warnings.

# Knife / Scissors

Affinity supports freehand/straight cuts and path breaks.[[9]](https://affinity.help/designer2/English.lproj/pages/Tools/tools_knife.html)

Petunia previews intersections and distinguishes Split Object from Break Path.

# Shape Builder

Affinity uses source selection, candidate regions, Add/Delete/Create modes, multiple drag methods and cleanup policies.[[10]](https://affinity.help/designer2/English.lproj/pages/Tools/tools_shapeBuilder.html)

Petunia adopts the candidate-region model but makes candidate versus committed state more visually distinct, exposes region count/outcome, previews cleanup and names Create Copy explicitly. Scope remains owned by 10.3.

# Vector Flood Fill

Affinity creates shapes from enclosed/overlapping regions and supports insertion and fill-stack modes.[[11]](https://affinity.help/designer2/English.lproj/pages/Tools/tools_vectorFloodFill.html)

Petunia shows whether the result is new geometry, appearance stack update or replacement.

# Gradient

Affinity supports fill/stroke target, multiple gradient/bitmap types, rotate/reverse and bitmap fitting.[[12]](https://affinity.help/designer2/English.lproj/pages/Tools/tools_gradient.html)

Petunia separates stop editing from gradient geometry, shows interpolation colour space and exposes exact stop position/opacity.

# Transparency

Affinity uses a dedicated transparency-gradient tool.[[13]](https://affinity.help/designer2/English.lproj/pages/Tools/tools_transparency.html)

Petunia keeps a dedicated mode but reuses the same stop editor component as Gradient for lower learning cost.

# Vector Brush

Affinity paints editable vector paths with brush imagery, width, opacity, stabilizer and input controllers.[[14]](https://affinity.help/designer2/English.lproj/pages/Tools/tools_Brush.html)

Petunia documents the full UX even while scope remains Post-V1 unless promoted.

# Artboard / Surface

Affinity Artboard adds, moves and resizes artboards.[[15]](https://affinity.help/designer2/English.lproj/pages/Tools/tools_artboard.html)

Petunia uses the broader Surface model and may expose dimensions, presets, bleed/layout metadata and content-move policy.

# Place

Affinity supports click-at-default-size or drag-to-size placement.[[16]](https://affinity.help/designer2/English.lproj/pages/Tools/tools_placeimage.html)

Petunia additionally makes Embed/Link policy and relink state explicit.

# Vector Crop

Affinity crop is non-destructive.[[17]](https://affinity.help/designer2/English.lproj/pages/Tools/tools_crop.html)

Petunia keeps non-destructive crop as default and reserves destructive trim/raster crop for explicit commands.

# Colour Picker

Affinity can sample current object or global content and average a radius.[[18]](https://affinity.help/designer2/English.lproj/pages/Tools/tools_clrpicker.html)

Petunia may add an expert HUD for semantic colour model/profile and sampled-value detail.

# Style Picker

Affinity samples subsets including stroke, fill, opacity, effects, character, paragraph and object settings.[[19]](https://affinity.help/designer2/English.lproj/pages/Tools/tools_stylePicker.html)

Petunia maps subsets to Property Schema groups and previews affected property groups before applying to heterogeneous selections.

# Measure / Area

Affinity provides transient distance, drawing-scale, area and perimeter measurements.[[20]](https://affinity.help/designer2/English.lproj/pages/Tools/tools_measure.html)[[21]](https://affinity.help/designer2/English.lproj/pages/Tools/tools_area.html)

Petunia treats measurement as transient by default; persistent annotations require explicit product scope.

# Shared fixture matrix

Each tool gets no-selection, valid-selection, mixed-selection, keyboard-only, HiDPI, extreme zoom, cancel, undo/redo, snapping on/off, degenerate geometry, large-document latency and headless/MCP parity fixtures where applicable.