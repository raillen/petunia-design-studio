# 22.3 — Workspace, Shortcut, Export & Tool Presets

# WorkspacePreset

DockTree, visible panels, toolbar layout, Persona association, density/theme overrides and window-role hints. Screen geometry normalized on restore.

# ShortcutProfile

ActionId→KeyChord mappings + platform base profile + conflict resolution metadata. Unknown ActionId retained disabled for plugin return where practical.

# ExportPreset

ExporterId/version + typed option values + fidelity policy + naming template + target scope defaults. Destination grants are not portable preset data.

# ToolPreset

Optional common parameter bundle for shapes/text/crop etc. Must reference stable PropertyIds/ToolId and be declarative.

# User UX

Save As, Update, Duplicate, Rename, Reset built-in, Import/Export. Built-ins immutable; user override references base rather than modifying package.

# Migration

When PropertyId/Exporter option deprecates, migrator maps or reports incompatible field. No silent drop.

# Tests

Cross-platform shortcut import, workspace with missing plugin panels, export preset exporter version change and theme/density restore.