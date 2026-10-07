# 18.32 — Clone Stamp Tool Specification

# Identity

Clone tool with source anchor.

# Source acquisition

Alt/Option-click sets source in chosen sampling scope. Source marker remains visible. If no source, pointer action does not paint and status prompts source selection.

# Aligned

Aligned mode keeps fixed offset between destination and source across strokes. Non-aligned restarts from original source each stroke. Exact behavior mirrors UI toggle.

# Sampling scope

Current Layer, Current & Below, All Layers. Composite source captured from coherent revision at stroke begin.

# Transform

Source sampling maps through layer/document transforms correctly; selected source layer identity retained where needed.

# Brush

Uses normal brush size/hardness/opacity/flow/stabilizer. Native engine samples source before destination write to avoid feedback artifacts unless deliberate current-layer evolving semantics specified.

# History

One stroke tile delta. Source point is tool state, not canonical document.

# Tests

aligned/nonaligned, transformed layers, sample merged, source overlapping destination, masks, tile seams and undo.