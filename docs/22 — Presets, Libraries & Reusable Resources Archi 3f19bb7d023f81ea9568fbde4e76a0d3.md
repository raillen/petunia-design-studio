# 22 — Presets, Libraries & Reusable Resources Architecture

# Unified model

Resource libraries cover BrushPreset, Swatch/Palette, GradientPreset, ObjectStyle, CharacterStyle, ParagraphStyle, Asset, SymbolLibrary, ExportPreset, WorkspacePreset and ShortcutProfile.

# Scopes

BuiltIn, User, Document, Plugin, ExternalLibrary. IDs are stable and namespace-aware.

# Contracts

Version, provenance, display metadata, semantic payload, dependencies, thumbnail, tags, compatibility and migration. Copy/link/reference behavior is explicit per resource kind.

# Conflict

Import handles same ID/different content by replace/copy/new ID/merge when semantic merge exists; never silent overwrite.

# Backup/share

User resources can export/import as packages excluding secrets/grants. Document resources necessary for fidelity can embed in PTND.

# Search

Unified indexing by kind/tags/name/favorites/recent, with lazy previews.

[22.1 — ResourceLibrary Core Model, IDs, Scope, Provenance & Versioning](22%201%20%E2%80%94%20ResourceLibrary%20Core%20Model,%20IDs,%20Scope,%20Pro%203f19bb7d023f8162907ff683c5a7f2a6.md)

[22.2 — Brush Preset & Gradient/Swatch Library Contracts](22%202%20%E2%80%94%20Brush%20Preset%20&%20Gradient%20Swatch%20Library%20Cont%203f19bb7d023f81a2818bd40513b72130.md)

[22.3 — Styles, Symbols & Asset Library Contracts](22%203%20%E2%80%94%20Styles,%20Symbols%20&%20Asset%20Library%20Contracts%203f19bb7d023f81918218c1792dc647e2.md)

[22.4 — Export, Workspace & Shortcut Preset Contracts](22%204%20%E2%80%94%20Export,%20Workspace%20&%20Shortcut%20Preset%20Contrac%203f19bb7d023f817083b2ff3c4f4e52dd.md)

[22.5 — Library Package Format, Import/Export, Conflict Resolution, Backup & Sync Boundaries](22%205%20%E2%80%94%20Library%20Package%20Format,%20Import%20Export,%20Conf%203f19bb7d023f811893a0efb33372f5fa.md)