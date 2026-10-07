# 18.01 — Move Tool Specification

# Identity

ToolId: ptnd.tool.move. Persona: Design+Photo. Default shortcut: V.

# Targets

Any selectable movable object according capability. Locked/read-only objects may be selectable but are not mutable.

# State machine

Idle -> HoverCandidate -> PressArmed -> DragTransform -> Commit/Cancel. Handle hover branches to Resize, Rotate, Shear or Pivot. Alt/Option before movement threshold may enter DuplicateTransform.

# Interaction

Click selects ranked candidate. Shift toggles. Drag begins transform after threshold. Empty drag starts marquee selection.

# Modifiers

Shift constrains axis/angle/aspect. Alt/Option duplicates or center-resizes by subgesture. Ctrl/Cmd uses deep/select-through policy from platform profile.

# Context

X, Y, W, H, anchor, aspect lock, rotation, shear, coordinate space, scale stroke/effects, snapping.

# Overlay

Bounding box, handles, pivot, smart guides, snap indicators and delta/size HUD.

# Commands

TransformObjects, DuplicateAndTransform, SetPivot if persistent.

# Undo

One gesture equals one logical transaction. Numeric field edits coalesce within one edit session.

# Performance

Tier R pointer-to-preview p99 <= 16.7 ms for representative selection; hit testing p95 <= 4 ms after spatial index warm.

# Accessibility

Arrow nudge and Transform panel provide non-drag precision alternatives.

# Edge cases

Rotated groups, negative scale, symbols, clipped children, multiple Surfaces, zero-size objects, singular transforms and locked ancestors.