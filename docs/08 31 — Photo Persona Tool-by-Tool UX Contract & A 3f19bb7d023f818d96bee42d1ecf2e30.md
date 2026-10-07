# 08.31 — Photo Persona Tool-by-Tool UX Contract & Affinity Pixel/Photo Mapping

# Companion

08.8 is the concise tool contract; 10.9–10.10 define engine semantics. This page records parity expectations against professional pixel/photo workflows.

# Required mappings

Pixel Brush, Eraser, Flood Fill, Gradient, Marquee variants, Lasso variants, Selection Brush, Refine, Crop/Straighten, Clone, Heal, Inpaint, Blemish, Dodge/Burn, Smudge/Blur/Sharpen, Picker, Hand/Zoom.

# For each mapping

ToolId, target types, source/sample policy, cursor, context controls, pen dynamics, modifier map, live overlay, cancel/commit, panel dependencies, Action/Command, nondestructive option, disabled reason, automation exposure, accessibility path and benchmark fixture.

# Photo UX invariants

Pixel target is never ambiguous; selection combine mode always visible; source point for Clone/Heal visible; destructive vs live filter explicit; expensive inpaint/refine is cancellable; histogram/analysis never blocks editing.

# Affinity divergence

Petunia may place clearer target/fidelity labels and expose automation-friendly semantic IDs; reference parity is capability/task-based, not icon/placement copying.