# 18.22 — Hand / Pan & Zoom Tools Specification

# Navigation state

Pan/zoom modify Viewport only; never Document revision/history.

# Hand

Space temporary override from any compatible tool; pointer drag pans. Middle mouse optional. Release temporary key restores exact prior tool/substate when safe.

# Zoom

Click zoom-in; modifier zoom-out; drag-zoom optional; wheel/pinch anchored near pointer. Fit Selection/Surface/All and 100% actions share View service.

# View transform

Pan/zoom stored double precision; clamp only to prevent unusable overflow. Extreme zoom bounds product-defined and testable.

# Canvas rotation

If view rotation supported, it is View state with reset action and does not rotate artwork.

# Accessibility

Keyboard +/-/fit and numeric zoom field.

# Tests

Temporary override during Pen/Node, high-DPI pointer anchoring, repeated zoom precision, multi-view independent viewport.