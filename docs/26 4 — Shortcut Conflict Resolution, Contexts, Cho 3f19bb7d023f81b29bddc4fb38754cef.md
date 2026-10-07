# 26.4 — Shortcut Conflict Resolution, Contexts, Chords & Profile Serialization

# Binding

ShortcutBinding = ActionId, KeyChord/sequence, context scope, platform profile, priority and user override source.

# Contexts

Global, Document, Canvas, TextEditing, ToolSpecific, PanelSpecific, Modal. Narrower context can legally reuse key when ambiguity is impossible and UI editor explains it.

# Chords

Single-step baseline. Multi-step chords can be enabled for power users if discoverability/timeout model is defined; they must not block ordinary text input.

# Conflict

Exact same context conflict is error. Prefix conflict in multi-chord mode is warning/error based timeout. OS-reserved key rejected or flagged unavailable.

# Serialization

ShortcutProfile stores only semantic overrides from base profile when possible, schemaVersion and platform applicability.

# Import/export

Unknown ActionIds preserved as inactive bindings so plugin/built-in action can return later.

# Tests

Text-vs-global, plugin action removed/returned, profile migration, conflict detection and locale keyboard layout.