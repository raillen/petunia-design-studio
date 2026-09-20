# 08.31 — Photo Persona Tool-by-Tool UX Contract & Affinity Pixel/Photo Mapping

<aside>
🖌️

**Photo tool UX uses the same interaction grammar as Design.** Brush-like tools share one predictable control vocabulary; selection and retouch tools expose source, target, preview and destructiveness explicitly.

</aside>

# Research baseline

Affinity Photo 2 documents selection, paint, erase and retouch tools as variants of a shared brush/context-toolbar grammar, while the 2026 Affinity Pixel Studio teaches crop/straighten, inpainting, global/targeted/painted adjustments and subtle filters as one progressive editing workflow.[[1]](https://affinity.help/photo2/English.lproj/)[[2]](https://www.canva.com/design-school/lessons/affinity-pixel-studio/)

# Shared brush grammar

All brush-derived tools use a stable control order where applicable:

1. brush preset;
2. width;
3. opacity;
4. flow;
5. hardness;
6. pressure/controller;
7. stabilizer;
8. source/target;
9. tool-specific mode;
10. advanced brush settings.

Width, opacity, flow and hardness must not jump to different locations between Paint, Clone, Heal, Selection Brush and Erase.

# Destructiveness banner

Before the first stroke, the tool state exposes one of:

- **Non-destructive on new/nested pixel layer**;
- **Writes to selected pixel layer**;
- **Requires rasterization**;
- **Preview-only until Apply**.

Affinity Paint Brush can create a nested pixel layer when painting an image/RAW layer, while its Inpainting behavior can rasterize an image/RAW layer depending on settings.[[3]](https://affinity.help/photo2/English.lproj/pages/Tools/tools_paintBrush.html)[[4]](https://affinity.help/photo2/English.lproj/pages/Tools/tools_inpaintingBrush.html)

Aubrieta must not hide this consequence behind an assistant preference.

# Selection Brush

Affinity Selection Brush paints Add/Subtract regions with width, edge snapping, all-layers sampling, soft edges and Refine.[[5]](https://affinity.help/photo2/English.lproj/pages/Tools/tools_selectionBrush.html)

Aubrieta:

- preserves Add/Subtract brush grammar;
- displays active selection source scope persistently;
- exposes edge-snap confidence/preview in developer inspection;
- offers Refine as a clear continuation, not a detached dialog surprise;
- shows marching-ants plus optional mask overlay without relying on animation alone.

# Flood Select

Affinity Flood Select uses New/Add/Subtract/Intersect, source, tolerance, contiguous, antialias and Refine; drag can change tolerance.[[6]](https://affinity.help/photo2/English.lproj/pages/Tools/tools_floodSelect.html)

Aubrieta adds a temporary numeric tolerance HUD during drag and keeps the pre-drag value recoverable on Esc.

# Marquee selections

Rectangle, ellipse, row/column and other supported marquee modes share New/Add/Subtract/Intersect semantics with Flood Select. Feather, antialias and fixed-size/aspect controls use the same PropertyField components used elsewhere.

# Quick Mask / selection visualization

Selection may be inspected as an overlay or temporary editable mask representation without changing its canonical transient/session status. Enter/exit state is visibly marked and Escape behavior is deterministic.

# Refine Selection

Refinement is a focused workflow with preview modes, edge width/border, smooth, feather, ramp/contrast and output target where supported. The output target must clearly distinguish Selection, Mask or new layer/object outcomes before Apply.

# Crop / Straighten

Affinity Crop supports non-destructive crop or resampling, aspect presets, DPI/units, straighten, composition overlays, reveal and explicit Apply/Cancel.[[7]](https://affinity.help/photo2/English.lproj/pages/Tools/tools_crop.html)

Aubrieta improvements:

- default remains non-destructive when document semantics permit;
- Resample is labeled as a pixel-changing operation;
- current resulting pixel dimensions are always visible when resampling;
- Straighten has live angle readout and reset;
- composition overlay is presentation state.

# Paint Brush

Affinity Paint Brush exposes width, opacity, flow, hardness, pressure, Rope/Window stabilizers, symmetry/mirror, blend mode, Wet Edges and Protect Alpha.[[3]](https://affinity.help/photo2/English.lproj/pages/Tools/tools_paintBrush.html)

Aubrieta:

- groups geometry, deposition, dynamics and compositing controls instead of a flat control strip;
- keeps brush cursor informative at extreme zoom;
- exposes latency/stabilizer state;
- makes Protect Alpha visually persistent;
- treats symmetry origin as transient workspace/tool state unless explicitly materialized.

# Erase

Erase shares brush grammar. Aubrieta differentiates destructive pixel erasing from mask painting with consequence-first labels and provides a direct action to switch to nondestructive mask editing where applicable.

# Clone

Affinity Clone uses a sample origin, optional Global Sources, aligned/un-aligned behavior, source scope, rotation, scale and flip with cursor preview.[[8]](https://affinity.help/photo2/English.lproj/pages/Tools/tools_cloneBrush.html)

Aubrieta:

- draws source-to-destination relation line while sampling/painting when useful;
- shows source thumbnail/name and layer/document origin;
- exposes aligned state beside source controls;
- previews rotate/scale/flip in cursor/HUD;
- warns when source becomes unavailable.

# Healing

Affinity Healing shares source mechanics with Clone but blends the sampled area into destination context.[[9]](https://affinity.help/photo2/English.lproj/pages/Tools/tools_healingBrush.html)

Aubrieta uses the same Source Picker component as Clone, then exposes the different semantic operation in tool name, icon and status hint.

# Inpainting

Affinity Inpainting restores/removes unwanted regions and may rasterize certain layer types.[[4]](https://affinity.help/photo2/English.lproj/pages/Tools/tools_inpaintingBrush.html)

Aubrieta treats this as an operation with an explicit target policy. If V1 implementation is destructive-only, UI offers Duplicate/Pixel Layer staging before the first destructive stroke rather than silently changing layer type.

# Dodge / Burn / Sponge / Blur / Sharpen / Smudge

These tools share brush controls but add a clear effect target such as tonal range, saturation behavior, strength or mode. Aubrieta groups them under Retouch and keeps tool-specific controls after the stable brush control block.

# Patch / Blemish

Patch-like tools make candidate source and target regions visible before commit. One-click blemish tools provide a hover preview footprint and remain undoable per atomic edit.

# Gradient

Photo Gradient uses the same semantic stop editor as Design where possible, but can target pixel/fill/mask context. Tool mode must identify whether it edits content, mask or adjustment parameter.

# Tool-target safety

Aubrieta always shows the active writable target near the context toolbar:

- layer name/type;
- mask versus pixels;
- linked/readonly status;
- bit depth/profile summary when relevant.

Attempting a destructive tool on an incompatible target produces a safe conversion/staging choice rather than an unexplained failure.

# Pen input

Brush tools support pressure/tilt where engine and hardware allow. Pressure mapping is inspectable and has a mouse-equivalent usable path. Pen barrel/eraser buttons never become the only way to access an essential action.

# Performance

Continuous pointer/pen feedback has a frame-bound preview path. Expensive recomputation is deferred without dropping stroke samples. Brush latency, tile invalidation and memory pressure are benchmarked on the Photo performance corpus.

# Canonical fixtures

Selection edge cases, selection refine, 16-bit tiles, transparent layer, mask target, linked image, large brush, pressure input, stabilizer, clone global source, missing source, destructive staging, undo one stroke, cancellation, 200% scale and mixed Design↔Photo workflow.