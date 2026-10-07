# 22.7 — Workspace, Theme, Shortcut & Accessibility Profiles

# WorkspaceProfile

DockTree, panel visibility, toolbar arrangement, Persona association, density override and optional monitor-layout hints. No document artwork state.

# ThemeProfile

References semantic token overrides within allowed schema, accent, dark/light base and contrast policy. Custom theme validation enforces minimum accessibility for chrome unless user explicitly enables unsafe developer mode.

# ShortcutProfile

ActionId -> KeyChord[] mappings, platform base, conflicts/resolution metadata. Tool temporary overrides separated from textual shortcut labels.

# AccessibilityProfile

Large handles/cursors, focus visibility, contrast, reduced motion, tooltip verbosity, keyboard navigation preferences and screen-reader verbosity.

# Portability

Profiles export without secrets, paths, grants or machine-specific monitor IDs unless optional sanitized mapping included.

# Merge

Import can replace, duplicate or merge shortcut mappings with conflict UI; workspace import maps unavailable PanelIds to placeholders.

# Tests

Cross-platform chord mapping, missing plugin panels/actions, profile migrations and theme accessibility validation.