# 11.2 — User-Facing Terminology, Localization Keys & Consistency Glossary

# Canonical vocabulary

Document, Surface, Persona, Workspace, Studio Panel, Layer, Group, Mask, Adjustment, Live Filter, Appearance, Fill, Stroke, Resource, Asset, Symbol, Style, Data Merge, Preflight.

# Localization

Visible strings use TextId and parameterized message templates. Never persist translated labels as identity.

# Terminology policy

One concept has one default term across menu/panel/help unless domain convention requires a distinct noun/verb. Avoid technical internals in UI: “render graph,” “DTO,” “nanobind”.

# Translation context

TextId metadata includes context/comment for ambiguous words, plural forms and shortcut mnemonic policy.

# Error copy

Message code stable; localized title/body can evolve. Technical detail optionally copyable.

# Glossary

Docs/user guide generated or cross-linked from canonical terminology table to keep Portuguese/English consistency.