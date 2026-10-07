# 05.10 — Accessibility Bridge: QAccessible, Semantic Canvas, Focus, Screen Readers & Keyboard Navigation

# Native widgets

Use correct Qt accessible roles/names/descriptions/states and labels-for relationships. Custom-painted controls implement QAccessibleInterface or expose equivalent accessible child model.

# Canvas semantics

Canvas itself is not represented as millions of pixels. Expose:

- current document/Surface;
- active tool;
- selection summary;
- selected object semantic children on demand;
- focused on-canvas handles/HUD;
- status/hints;
- actionable overlay controls.

# Focus

FocusManager tracks zones: chrome, tool rail, canvas, docks, dialogs. Keyboard commands cycle panels and return focus to canvas predictably.

# Screen reader updates

Announce meaningful changes: tool activated, target changed, selection count, operation error, snap target if user enables verbose spatial feedback. Avoid flooding pointer move.

# Custom graphs

Curves/pressure editors have accessible table/list representation and keyboard movement.

# High contrast

Token theme integrates OS preference where available; document artwork unaffected.

# Testing

QAccessible inspection + external screen-reader matrix. Semantic MCP test tree can share IDs but cannot substitute platform API testing.