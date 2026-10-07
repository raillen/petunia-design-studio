# 00.5.2 — Keyboard-First, Screen Reader & Non-Visual Canvas Interaction Contract

# Keyboard

Every essential workflow has menu/palette/action route. Drag operations have numeric/command alternatives where precision function exists.

# Focus zones

Tool rail, canvas, context bar, dock groups, status/job center and dialogs can be traversed predictably; focus return after dialog/tool edit is deterministic.

# Semantic canvas

Expose active Surface, tool, selection count/objects, focused handle/HUD, snap/measurement feedback and available contextual actions—not raw pixel tree.

# Object navigation

Optional semantic next/previous object and layer-tree focus let non-visual users inspect/select without hit-testing coordinates.

# Announcements

Tool activation, selection count, pixel target, destructive/live mode, error and optionally snap target. Pointer-hover noise is suppressed.

# Graph controls

Curves, brush dynamics and gradients expose list/table numeric editing alongside visual graph.

# Testing

Keyboard-only golden flows and screen-reader matrix with NVDA/JAWS/VoiceOver/Orca as supported platforms permit.