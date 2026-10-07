# 22.4 — Export, Workspace & Shortcut Preset Contracts

# ExportPreset

ExporterId, target format/profile, dimensions/scales, resampling, color conversion, metadata, area policy, fidelity policy, filename suffix/template and advanced format options.

# Compatibility

Preset declares required exporter capability/version. Opening missing/old exporter preserves preset but marks unavailable/incompatible.

# WorkspacePreset

DockTree template, panel visibility, Persona association, toolbars, density/theme override, monitor topology hints. No document data/secrets/grants.

# ShortcutProfile

ActionId -> one or more KeyChord bindings, platform base profile, conflict metadata and user overrides. Labels are not identity.

# Built-in vs user

Built-ins immutable; user Duplicate creates new ID. Update built-in app version can add/change defaults without overwriting explicit user profile.

# Sharing

Export/import package strips machine-specific screen geometry, secrets and file grants unless explicitly portable field.

# Tests

Exporter unavailable, monitor topology change, shortcut conflicts, profile migration and built-in update preserving user overrides.