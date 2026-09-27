# 09.25 — Property & Parameter Schema Registry, Generic Inspectors & Bindable Data Contracts

# Why

A modular creative suite cannot hard-code every Properties panel, MCP schema, Data Merge target and plugin parameter separately. Aubrieta needs a **reflection-free semantic schema registry**.

# IDs

`PropertyId`, `ParameterId`, `SchemaId` are namespaced extensible IDs. Built-ins get generated typed constants; plugins can register new namespaces.

# PropertyDescriptor

Declares: TextId label/description, optional IconId, value type, unit/dimension, editor hint, range/step/precision, default, nullable/mixed semantics, read-only predicate, selection applicability, invalidation class, serialization key/version, bindable flag, automation visibility and module owner.

# Value types

Boolean, integer, float/scalar+unit, enum, string, ColorValue, resource reference, object reference, vector/point/rect/transform, gradient/stroke/effect reference and structured custom schema. Avoid `serde_json::Value` as universal in-memory domain type; use typed values with schema-aware serialization at boundaries.

# Access

UI/MCP never gets a mutable pointer to object property. `PropertyProvider` reads a snapshot/value for a selection and produces a typed Command request for edits. Multi-selection returns Same/Mixed/Unavailable states.

# Parameter schemas

Tools/effects/adjustments/exporters reuse descriptors. Generic inspectors can render common controls; complex custom editor registered by semantic `EditorId` may override presentation while using same property commands.

# Data Merge

Only descriptors with `bindable=true` can be bound. They define accepted source types and coercion/formatter rules.

# Automation

MCP discovery emits stable schema from descriptors. Localized TextId is metadata, never identity. Plugin parameter schemas are namespaced and capability-validated.

# Versioning

Persisted custom parameter blocks include schema namespace/version. Migration owner module provides upgrades. Missing provider retains opaque data.

# Tests

Mixed selections, invalid type/range, plugin schema collision, custom editor fallback to generic inspector, bindability enforcement, schema migration, MCP roundtrip and module disable.

# Schema registry ownership

`SchemaRegistry` is application/foundation infrastructure and stores immutable validated descriptors contributed by built-ins/plugins/modules. It is not a UI registry. GPUI, MCP, Data Merge and plugins consume the same semantic descriptor snapshot.

Each registered schema has:

- stable `SchemaId`;
- owner `ModuleId`/provider;
- semantic version;
- value/object applicability domain;
- property/parameter descriptors;
- optional migration hooks/metadata;
- contribution capabilities/permissions;
- documentation/help topic metadata.

Registration is transactional: a schema with duplicate IDs, invalid ranges/types, unknown editor requirements or namespace collision is rejected entirely rather than half-published.

# Typed PropertyValue

Avoid universal JSON values in the hot/runtime API. Canonical conceptual union:

```
PropertyValue
├── Bool
├── Int(i64)
├── Float(FiniteF64)
├── String
├── Enum(EnumValueId)
├── Length/Angle/Percent/Duration typed scalar
├── Point/Size/Rect/Transform
├── Color(ColorValue)
├── Resource(ResourceId)
├── Object(ObjectId)
├── Gradient/Stroke/Effect semantic value/reference
├── List<TypedValue> with bounded schema
└── Struct { SchemaId, fields: typed values }
```

Boundary serializers may map this to JSON/MCP/Lua values using schema, but core callers do not guess types from JSON shapes.

# Unit/dimension model

Numeric descriptors declare semantic dimension separately from current display unit:

- unitless scalar;
- length;
- angle;
- percentage/normalized scalar;
- pixel/image dimension;
- resolution;
- time/duration;
- color-channel/model-specific numeric domain.

The descriptor owns allowed unit families and canonical internal representation. UI/localization may display mm/in/pt/etc.; MCP can submit a typed `{value, unit}` or canonical numeric form documented by schema. Conversions never depend on localized strings.

# Value state for selection

Property query returns explicit state:

```
PropertyState
├── Same { value, source?, default?, editable }
├── Mixed { representative_metadata, editable }
├── Unavailable { reason_code }
├── ReadOnly { value?, reason_code }
├── Loading/Stale { previous_value?, generation }
└── Error { diagnostic }
```

Do not encode Mixed as an empty string/NaN/null unless the property itself is nullable. `null` is a legitimate semantic value only when descriptor declares it.

# Multi-selection applicability

Descriptor defines an applicability function/predicate over semantic object kinds/capabilities. Query for a selection computes:

- property visible if it is meaningful to at least the policy-defined selection domain;
- editable common property only when every targeted object supports compatible write semantics, unless command explicitly supports partial-target edits;
- object-specific sections remain separate rather than coercing unrelated values into one generic editor.

Default V1 common-edit policy is **all selected supported targets or reject**, not silent partial edits. A specialized Action may explicitly offer “Apply to supported objects only” with preflight/affected count.

# Property paths and identity

Nested structured properties use stable semantic path segments/IDs, not display labels or Rust field names. Example:

```
aubrieta.object.transform.position.x
aubrieta.appearance.stroke.width
aubrieta.text.paragraph.leading
```

A path is schema-versioned semantic identity. Renaming a Rust field does not change PropertyId; changing semantic meaning requires migration/new property ID.

# Read/write ports

Recommended shape:

```
PropertyQueryRequest {
  document_session_id,
  view/selection or explicit target IDs,
  property IDs/filter,
  revision
}

PropertyEditRequest {
  explicit target IDs,
  PropertyId,
  typed value/edit operation,
  expected_revision?,
  edit policy,
  origin/action/correlation
}
```

Edits compile to validated Commands. Provider cannot return a closure that mutates objects directly.

# Edit operations

Not every edit is `set(value)`. Descriptor may support typed operations:

- `Set`;
- `ResetToDefault`;
- `ClearOverride`;
- `Increment/Adjust` for numeric expert controls;
- `Add/Remove/Reorder` for schema-owned list structures;
- `Bind/Unbind` through DataBinding contract;
- `AssignResource`.

Each operation declares undo/invalidation/permission semantics.

# Validation stages

Before command creation:

1. schema/property exists and provider active;
2. target applicability;
3. value type/schema match;
4. finite/range/unit/enum/reference validation;
5. contextual constraints depending on document/selection;
6. permission/capability;
7. command-level domain invariant validation.

UI-side validation improves feedback but never replaces host/domain validation. MCP/plugin receives same structured failure codes.

# Range/precision semantics

`min/max/step/precision` are presentation/editing hints unless descriptor explicitly marks them as hard domain constraints. Separate:

- `HardConstraint` — command rejects outside range;
- `SuggestedRange` — slider/editor range, typed input may exceed;
- `DisplayPrecision` — formatting only, never rounds canonical value unless user commits rounded value.

This avoids a slider's visual range accidentally becoming document semantics.

# Enum registry

Enums use stable `EnumValueId`, each with TextId/IconId/order/deprecation metadata. UI does not persist localized label. Plugins may extend only enum domains explicitly marked extensible; closed semantic enums reject foreign values.

# Defaults and inherited values

Descriptor distinguishes:

- intrinsic schema default;
- document/style inherited value;
- object local override;
- mixed selection.

Property query may return provenance metadata (`Local`, `Style(StyleId)`, `Inherited`, `Default`) so reset/revert UI can behave correctly. Reset operations target provenance semantics, not simply assign a hard-coded default literal.

# Visibility/read-only predicates

Predicates are **declarative semantic expressions/callbacks inside trusted owner module**, evaluated against bounded property/context snapshots. Plugin declarative schemas use a restricted expression model and cannot execute arbitrary code during every Properties layout pass.

Predicates expose dependency PropertyIds so the inspector knows when to recompute. Hidden/disabled reasons use stable reason/TextIds.

# Editor hints

`EditorHint` is presentation metadata, not toolkit type. Examples:

- numeric field/slider/scrubber;
- color picker;
- resource picker;
- enum segmented/dropdown;
- gradient/stroke specialized semantic editor;
- multiline text;
- matrix/origin control.

A shell may ignore a hint and use a valid generic editor. A custom `EditorId` must declare a generic fallback unless the feature cannot be safely edited generically, in which case Unavailable reason is explicit.

# Generic inspector generation

Inspector groups descriptors by semantic section/order metadata and selection applicability. Generated UI receives:

- descriptors;
- PropertyState values;
- edit operations/action IDs;
- TextId/IconId/help IDs;
- diagnostics/disabled reasons.

It does not receive concrete object mutation callbacks.

# Property delta/update model

Presentation layers subscribe by target/schema/property interest and receive `PropertyDelta` keyed by document revision/presentation generation. Deltas may invalidate specific properties/sections. Consumers can always requery a full snapshot as correctness fallback.

# Caching

Schema descriptors are immutable/versioned and cheap to cache. Property values derived from selection/object state are generation-bound; never retain stale values after ChangeSet affecting their dependencies. Expensive properties may expose `Loading/Stale` and compute through JobSystem.

# ParameterDescriptor distinction

A `ParameterDescriptor` describes invocation/tool/effect/export parameters that may not correspond to a persisted object property. It reuses common value/schema/unit/validation metadata but declares:

- required/optional;
- invocation default;
- positional ordering only for generated bindings if needed;
- whether it is interactive-previewable;
- serialization relevance;
- side-effect/permission implications.

Do not force every command parameter into PropertyId solely to reuse UI code.

# Tool parameter lifecycle

Tool settings such as brush size may live in tool/session preferences until committed into an object/stroke semantic result. Descriptor declares scope (`ToolSession`, `UserPreference`, `DocumentProperty`, etc.) so generic UI knows where edits go.

# Effect/adjustment schemas

Effect type contributes:

- `EffectTypeId`;
- parameter schema version;
- typed parameters/defaults;
- evaluation invalidation classes;
- capability/bit-depth/color requirements;
- generic/custom editor metadata;
- migration owner.

Persisted effect instance records type ID + schema version + parameter values. Missing provider preserves opaque instance and render/preflight fallback state.

# Data Merge binding contract

A bindable PropertyDescriptor includes `BindingDescriptor`:

- accepted DataField types;
- permitted coercions;
- formatter categories;
- null/missing policy options;
- whether binding changes structure/resource reference/appearance;
- validation/preflight requirements.

No generic string conversion fallback. Text→Color or number→resource requires an explicitly registered safe converter.

# Binding evaluation

Data binding produces a derived/evaluated property overlay for a preview record; it does not rewrite the literal canonical property for each preview. Materialize/generate is an explicit command/output process. Property query in Data Preview Mode can expose literal value + bound/evaluated value separately to avoid confusion.

# MCP schema projection

MCP discovery projects descriptors into protocol schemas without losing semantic type/unit/enum IDs. Localized labels remain metadata. Large registry discovery is filterable/pageable by owner/domain/PropertyId prefix.

MCP edits should prefer typed property methods/batch operations and `expected_revision`. Invalid value returns schema path + expected type/range/allowed IDs + recoverability.

# Lua plugin binding

Lua binding converts `PropertyValue` using descriptor rules. Semantic IDs are strings/userdata wrappers; enums are stable IDs; resource/object references are opaque IDs. Lua table shape for structured values is generated/documented from schema, not handcrafted differently per plugin API.

# Schema compatibility/versioning

Schema major/minor policy:

- additive optional property/enum metadata can remain compatible;
- changing value type, unit semantics, required field meaning or persisted interpretation is breaking for that schema and requires migration/new major schema version;
- deprecating a PropertyId keeps discovery metadata/replacement until removal window;
- persisted schema versions migrate stepwise through owner module.

# Missing provider

If a persisted custom schema/provider is missing:

- payload remains opaque/versioned;
- generic UI may show read-only metadata if host knows safe descriptor snapshot from manifest/schema cache;
- no command edits unknown semantics;
- reactivation restores interpretation after migration checks;
- document can round-trip without data loss under opaque preservation limits.

# Schema introspection documentation

Generate VitePress/reference pages from registry metadata:

- IDs/types/units/defaults;
- enum values;
- bindability/coercions;
- automation visibility;
- permissions;
- examples;
- version/deprecation.

Code examples are compiled/tested against the same generated schema.

# Registry quality gates

CI validates:

- unique namespaced IDs;
- descriptor owner namespace;
- hard constraints/default consistency;
- TextId/IconId/help existence;
- generic editor available where required;
- serialization key/version for persisted property;
- bindable converters exist;
- MCP-visible properties are serializable/projectable;
- plugin-visible schemas obey permission/data-size limits;
- no GPUI/third-party engine type appears in descriptor public contracts.

# Required gauntlets

- 1000+ descriptors discovery/query performance;
- mixed multi-selection across compatible/incompatible object types;
- hard vs suggested range distinction;
- localized unit/display roundtrip with invariant canonical value;
- plugin schema disable/re-enable and opaque persistence;
- schema version migration;
- same edit through GPUI generic inspector, MCP and Lua produces identical Command outcome;
- Data Merge preview binding does not mutate literal property;
- custom editor absent falls back generically;
- stale property delta never overwrites newer revision presentation.