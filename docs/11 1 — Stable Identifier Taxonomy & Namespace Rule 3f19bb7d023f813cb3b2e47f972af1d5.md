# 11.1 — Stable Identifier Taxonomy & Namespace Rules

# Stable IDs

ActionId, ToolId, PanelId, PropertyId, EffectKind, ImporterId, ExporterId, CapabilityId, TextId, IconId, HelpId and plugin contribution IDs are machine identities.

# Namespace

Built-ins use ptnd.<domain>.<name>. Third-party uses reverse-DNS/plugin-owned prefix such as [org.example.plugin.action.foo](http://org.example.plugin.action.foo). IDs lowercase ASCII with dots and underscores/hyphens only per exact regex chosen in schema.

# Stability

Cosmetic rename/localization never changes ID. Breaking semantic replacement introduces new ID and deprecation mapping.

# Object IDs

Runtime/document entity IDs are UUID-like opaque typed values and not human namespace strings.

# Collision

Registry rejects duplicates deterministically and reports provider/source.

# Tests

Schema regex, duplicate registration, deprecated alias and localization independence.