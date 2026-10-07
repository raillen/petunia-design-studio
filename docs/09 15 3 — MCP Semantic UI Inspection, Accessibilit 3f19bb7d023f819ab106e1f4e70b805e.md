# 09.15.3 — MCP Semantic UI Inspection, Accessibility Tree, Controlled Input & Test Mode

# Why semantic

Agents should not rely on coordinates/screenshots when application already knows ToolId, ActionId, PanelId and control semantics.

# UI tree

Window -> workspace -> toolbar/panel/canvas HUD/dialog -> semantic controls. Each node: stable test/semantic ID, role, label, value/state, enabled/visible/focused, bounds optional, associated ActionId/PropertyId.

# Canvas state

Active Surface, viewport transform, selection IDs/bounds, active tool, tool state, semantic overlay/hit targets that are safe to expose. Artwork pixels remain separate.

# Panel inspection

Visible/docked/floating panels, active tab, search/filter state if relevant, selected rows/object IDs, property controls and mixed/error states.

# Accessibility convergence

Inspection can reuse richer internal semantic layer feeding QAccessible, but test/MCP IDs remain stable technical identifiers.

# Screenshot

Developer/test-only scope. Capture identifies window/region, DPI/theme and revision. It supplements semantic evidence.

# Synthetic input

Developer/test-only pointer/key/pen simulation. Preferred functional automation remains Action/Command. Synthetic input exists to verify actual interaction grammar and focus/cursor behavior.

# Safety

Production remote clients cannot click arbitrary OS UI or interact outside Petunia window through this API.

# Tests

Semantic tree under dark/light, localization, disabled actions, modal dialogs, plugin panels and mixed DPI; compare synthetic gesture canonical result to direct Action path.