# 08.36 — Qt Interface Conformance Gauntlet, Interaction Manifest & Release Gates

# Manifest

Machine-readable catalog should enumerate ToolId, PanelId, ActionId, dialog/window IDs, default shortcuts, context controls, accessible names and owning module.

# Static conformance

No duplicate IDs, missing TextId/IconId, illegal shortcut conflicts, hard-coded forbidden tokens, panels without provider metadata.

# Runtime conformance

Instantiate every panel/dialog/component in test shell; activate every tool; verify no missing provider/schema, focus trap or uncaught exception.

# Interaction

Replay canonical gestures via semantic/test input and compare Commands/canonical result. Test cancel/lost pointer capture/window deactivate.

# DPI/theme

Full suite samples critical UI in Dark/Light, Compact/Comfortable, 100/150/200%.

# Release blockers

Missing essential semantic action, inaccessible critical flow, broken save/export, destructive conversion without explicit label, workspace corruption, unhandled plugin panel disappearance or persistent uncaught UI exception.