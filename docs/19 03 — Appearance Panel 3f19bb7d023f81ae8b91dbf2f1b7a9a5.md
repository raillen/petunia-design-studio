# 19.03 — Appearance Panel

# Identity

PanelId ptnd.panel.appearance.

# Model

Ordered AppearanceEntry list for primary selection: fills, strokes, effects and object-level opacity/blend summary.

# Entry row

visibility toggle, type icon, swatch/preview, descriptive name, badges (global swatch/live effect), expand arrow for parameters where appropriate.

# Selection

Selecting entry establishes AppearanceTarget used by Color, Stroke, Gradient and Properties. Multi-object selection can show compatible common entry model only under explicit matching rules.

# Operations

Add Fill, Add Stroke, Add Effect; duplicate; remove; reorder; enable/disable; reset; copy/paste appearance; expand/bake.

# Drag

Reorder entries with legal placement preview. Effects/fills/strokes ordering semantics enforced by core; invalid move rejected with reason.

# Synchronization

Color/Stroke panel edits selected entry via same EntryId. No feedback-loop duplicate commands.

# Accessibility

List semantics, entry type/state, reorder via keyboard actions.

# Tests

Multiple fills/strokes, effect reorder, mixed selection, deleted EntryId while panel active, undo and export preflight linkage.