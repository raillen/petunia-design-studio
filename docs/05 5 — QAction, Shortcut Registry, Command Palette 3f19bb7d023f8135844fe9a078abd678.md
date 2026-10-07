# 05.5 — QAction, Shortcut Registry, Command Palette & Context-Sensitive Availability

# ActionAdapter

For each ActionDescriptor create/update QAction with localized title, icon, default shortcut, checkable state and help. QAction.triggered invokes ActionDispatcher by ActionId.

# Availability

ActionAvailabilityService evaluates active session, Persona, tool, selection capabilities, permissions and modal state. It returns enabled/visible/checked/disabledReason.

# Shortcut registry

Default chords are platform profile data. User mappings override. Conflicts are detected before activation and surfaced in editor.

# Context

No widget asks arbitrary parent chain to know active document. WindowContext/ActionContext is injected/resolved explicitly.

# Command palette

Indexes descriptors, aliases, tags, panels/settings/help. Execution path is identical to QAction.

# Tool shortcuts

Tool activation is ActionId. Temporary override keys use InputModeStack and do not replace shortcut registry with ad-hoc event filters.

# Test

For every ActionId, test menu/shortcut/palette dispatch creates same semantic request/result.