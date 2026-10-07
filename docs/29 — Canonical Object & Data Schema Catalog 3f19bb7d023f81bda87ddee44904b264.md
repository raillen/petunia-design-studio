# 29 — Canonical Object & Data Schema Catalog

# Purpose

Define canonical semantic records independently from PTND JSON syntax and C++ class layout.

# Scope

Document, Surface, Object common fields, Group, VectorPath, ParametricShape, RasterLayer, TextObject/Story, Mask, Adjustment, LiveFilter, LiveBoolean, Symbol, Style, Resource, DataSource/Binding and extension payload.

# Each schema

Stable type ID; required/optional fields; value types/units; invariants; references/ownership; serialization mapping; mutation Properties/Commands; derived data; capability/version; validation errors and migration notes.

# Rule

C++ classes may be optimized differently, but observable semantics cannot diverge from catalog without schema/ADR update.

[29.1 — Document, Surface, Hierarchy & Common Object Record Schemas](29%201%20%E2%80%94%20Document,%20Surface,%20Hierarchy%20&%20Common%20Objec%203f19bb7d023f813f88dbfa9e929d2e9d.md)

[29.2 — VectorPath, ParametricShape, LiveBoolean & Appearance Schemas](29%202%20%E2%80%94%20VectorPath,%20ParametricShape,%20LiveBoolean%20&%20%203f19bb7d023f8166a06cfd4f586a44c9.md)

[29.3 — RasterLayer, TileSet, PixelMask, Selection & Channel Schemas](29%203%20%E2%80%94%20RasterLayer,%20TileSet,%20PixelMask,%20Selection%20%203f19bb7d023f81ffbf69f6a0209b5dd8.md)

[29.4 — TextStory, TextObject, Styles, Flow & Inline Field Schemas](29%204%20%E2%80%94%20TextStory,%20TextObject,%20Styles,%20Flow%20&%20Inlin%203f19bb7d023f81999d2afa807b54eab3.md)

[29.5 — Adjustment, LiveFilter, Mask, Symbol, Style & Resource Schemas](29%205%20%E2%80%94%20Adjustment,%20LiveFilter,%20Mask,%20Symbol,%20Style%203f19bb7d023f81559ba3f8a3f0147612.md)

[29.6 — DataSource, Binding, ExportTarget, ExtensionPayload & Metadata Schemas](29%206%20%E2%80%94%20DataSource,%20Binding,%20ExportTarget,%20Extensio%203f19bb7d023f8144ad5dd20e21815a90.md)

[29.7 — Schema Validation Order, Error Codes, Migrations & Generated Type Bindings](29%207%20%E2%80%94%20Schema%20Validation%20Order,%20Error%20Codes,%20Migra%203f19bb7d023f81e0a8e1e8d607178dfe.md)