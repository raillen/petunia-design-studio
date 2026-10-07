# 26 — Default Shortcuts, Input Profiles, Modifiers & Accessibility Alternatives

# Purpose

Freeze default input grammar as a versioned profile, while allowing user remapping.

# Profiles

Windows/Linux default and macOS default share semantic ActionIds but differ Cmd/Ctrl conventions. Pen/tablet buttons and accessibility profiles are overlays.

# Principles

One ActionId can have multiple chords. Essential actions remain reachable through menus/command palette. Tool temporary modifiers never conflict silently with text editing/IME.

# Categories

File/Document; Edit/History; Selection/Transform; Tools Design; Tools Photo; View/Navigation; Panels/Workspace; Text; Layer/Arrange; Export; Accessibility.

# Conflict

ShortcutRegistry detects exact and prefix conflicts, context conflicts and OS-reserved combinations. User editor explains scope and winning binding.

# Temporary tools

Space Hand; temporary Eyedropper and modifier behaviors are InputModeStack states, not ordinary persistent shortcuts.

# Accessibility

Provide keyboard-first profile with direct panel focus, canvas semantic navigation, larger nudge steps and reduced reliance on modifier-drag.

# Versioning

Built-in profile has version. App upgrades add defaults without overwriting explicit user bindings.

[26.1 — Windows/Linux Default Shortcut Profile](26%201%20%E2%80%94%20Windows%20Linux%20Default%20Shortcut%20Profile%203f19bb7d023f8198b6deeffaa24185c4.md)

[26.2 — macOS Default Shortcut Profile](26%202%20%E2%80%94%20macOS%20Default%20Shortcut%20Profile%203f19bb7d023f817d9c54ff9db6c7b67d.md)

[26.3 — Pen/Tablet Button Mapping, Temporary Modes & Device Profiles](26%203%20%E2%80%94%20Pen%20Tablet%20Button%20Mapping,%20Temporary%20Modes%20%203f19bb7d023f81a38d90db22ac79d8d6.md)

[26.4 — Shortcut Conflict Resolution, Contexts, Chords & Profile Serialization](26%204%20%E2%80%94%20Shortcut%20Conflict%20Resolution,%20Contexts,%20Cho%203f19bb7d023f81b29bddc4fb38754cef.md)