# 18.19 — Hand / Pan Tool

# Identity

ToolId ptnd.tool.hand. Space temporary override; middle mouse optional.

# Purpose

Navigate viewport only.

# Interaction

Drag pans in logical screen space mapped to viewport transform. Kinetic/inertial scrolling optional for touch/trackpad and disabled by Reduce Motion preference if appropriate.

# Temporary override

Holding Space pushes Hand onto InputModeStack; releasing restores exact prior tool/substate without committing/cancelling its safe paused gesture unless tool contract forbids.

# Constraints

Pan has no document mutation/history. Clamp policy allows pasteboard overscroll to keep edge content centered.

# Multi-input

Trackpad two-finger pan and touch gesture can feed same ViewportController.

# Accessibility

Keyboard pan arrows/Page keys when canvas navigation focus; Navigator panel provides alternate.

# Performance

Viewport update and frame request no Python object traversal.

# Tests

Temporary override mid Pen construction, high zoom, overscroll, trackpad and focus loss.