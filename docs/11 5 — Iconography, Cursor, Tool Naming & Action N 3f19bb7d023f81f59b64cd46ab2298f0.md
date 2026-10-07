# 11.5 — Iconography, Cursor, Tool Naming & Action Naming Contract

# Icons

Every built-in Tool/Panel/Action can have IconId independent from file path. SVG assets follow common optical grid/stroke/fill conventions and theme-state rules.

# Tool names

Prefer established professional vocabulary when it improves transfer learning: Move, Node, Pen, Crop, Clone, Curves. Petunia-specific term only where model genuinely differs.

# Actions

Verb-first user labels: Create Mask, Expand Stroke, Convert to Curves, Rasterize. ActionId remains namespaced semantic verb.

# Destructive actions

Name the conversion/loss explicitly. Avoid generic “Apply” where result changes editability.

# Cursors

CursorId distinct from ToolId because one tool has multiple interaction cursors. Hotspot/DPR variants tested. Cursor is supplemental status, never sole indicator.

# Search aliases

Command palette can index Photoshop/Affinity/Illustrator common aliases as discoverability metadata without changing canonical terminology.