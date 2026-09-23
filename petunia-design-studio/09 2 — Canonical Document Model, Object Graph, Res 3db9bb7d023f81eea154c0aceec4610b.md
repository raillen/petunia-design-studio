# 09.2 — Canonical Document Model, Object Graph, Resources & IDs

# Document truth

`DocumentStore` owns persistent creative semantics only. Session/UI/render caches remain outside.

# Root structure

Conceptually: `DocumentMetadata`, `DocumentColorContext`, `ResourceCatalog`, `SurfaceStore`, `ObjectStore`, `StyleCatalog`, `SymbolCatalog`, `DataSourceCatalog`, `ExtensionData`.

# Object hierarchy

Objects use stable typed IDs and explicit parent/child relationships. **V1_REQUIRED baseline object kinds:** Path, ParametricShape, Text, ImageObject, PixelLayer, Group/container roles, CompoundPath, BooleanGroup, ClipGroup, Mask, SymbolInstance and Adjustment/Effect carriers. Forward-compatible unknown extension/object payloads use the opaque-data preservation contract; new first-class object kinds require an explicit schema/capability status rather than an unclassified “future object” placeholder.

Every object has common metadata: ID, name, visibility, lock state, transform, opacity/blend/isolation, parent, ordered children where applicable, style/effect references and extensible metadata.

# Resource model

Resources are referenced by ID, never embedded ad hoc in object structs. Images, fonts, ICC profiles, gradients, swatches, brushes, symbols, external links and data sources define ownership/link/embed state and content fingerprint.

# Stable IDs

Use typed newtypes over generational/UUID-like persistent identity. Runtime slot handles may optimize lookup but are never serialized identity. Deleted IDs are not recycled within a document session in ways that make stale references valid.

# Invariants

- object has at most one structural parent;
- tree is acyclic;
- references may form graphs but must be validated;
- Surface owns root object ordering, not object identity;
- resource deletion with live references becomes reject/replace/orphan workflow, never dangling memory access;
- symbol cycles are rejected or broken by defined rules;
- masks/clips have explicit attachment semantics.

# Unknown data preservation

Forward-compatible loaders preserve unknown extension/object payloads as opaque blocks when safe, enabling round-trip without understanding them.

# Validation

Document validation runs after load/migration and in debug/test mutation paths. Structural corruption yields diagnostics with object/resource IDs and repair strategy.

# Implementation contract — storage topology

The canonical in-memory representation is intentionally simple and inspectable rather than pointer-rich:

```
DocumentStore
├── metadata
├── color_context
├── surfaces: OrderedMap<SurfaceId, SurfaceRecord>
├── objects: Map<ObjectId, ObjectRecord>
├── resources: Map<ResourceId, ResourceRecord>
├── styles / symbols / data_sources
└── extension_data

SurfaceRecord.root_children: Vec<ObjectId>
ContainerObject.children: Vec<ObjectId>
ObjectRecord.parent: ParentRef = Surface(SurfaceId) | Object(ObjectId)
```

`ObjectRecord` is owned by the store exactly once. Parent/child collections contain IDs, never nested owned objects. This avoids recursive ownership coupling, makes IDs stable across moves/reparenting and allows validation/recovery without unsafe pointer repair.

# Persistent identity vs runtime lookup

Canonical recommendation:

- persistent IDs are typed 128-bit UUID-backed newtypes (`DocumentId`, `ObjectId`, `SurfaceId`, `ResourceId`, etc.); V1 should use UUIDv7 generation where available so IDs remain globally unique while retaining useful creation locality;
- runtime acceleration may map persistent IDs to `slotmap`/generational handles or dense indices inside derived stores;
- runtime handles are never serialized, exposed through plugin/MCP public contracts as identity, or retained across document reload;
- an ID created and then deleted is tombstoned for the current `DocumentSession`; it is never reissued in that session;
- undo may resurrect the same persistent ID because it reverses deletion rather than creates a new semantic object;
- copy/duplicate creates new IDs recursively while preserving explicit references only according to copy policy.

# Canonical ordering policy

Sibling/z-order is represented by ordered ID sequences owned by the Surface or container. V1 deliberately prefers `Vec<ObjectId>` semantics over linked-list/order-key/CRDT complexity. Reorder operations are transactional and update the smallest owning sequence. If profiling later proves very large sibling lists problematic, storage may change behind the same ordering contract without changing the file schema.

# Object common header

Every object record must expose a common header independent of concrete payload:

```
ObjectHeader
- id: ObjectId
- type_id: ObjectTypeId
- name: Optional<UserLabel>
- parent: ParentRef
- visible: bool
- locked: bool
- transform: Transform2D
- opacity: NormalizedScalar
- blend_mode: BlendMode
- isolation: IsolationMode
- appearance: AppearanceRef/inline AppearanceStack
- effect_chain: Vec<EffectInstanceId>
- metadata: bounded namespaced metadata
```

Type-specific payload follows the header. Feature modules must not duplicate common visibility/transform/appearance fields inside private payloads.

# Ownership and mutation rule

`DocumentStore` offers read-only queries to ordinary consumers. Mutation is accessible only to `DocumentMutator`/validated command internals. Renderers, presentation models, plugins, MCP, importers after parse, background jobs and exporters never receive mutable store references.

A mutation that changes parentage must atomically update both sides of the relationship and preserve child order. There is no observable state in which `child.parent != owner-of-child-id-list`.

# Reference classes

Every reference field must be classified in schema metadata as one of:

1. **structural** — parent/children; must always resolve after validation;
2. **strong semantic** — required resource/style/symbol dependency; deletion requires reject/replace/orphan policy;
3. **weak semantic** — optional link whose missing target has a defined fallback;
4. **external** — URI/path/source descriptor resolved through platform/resource services;
5. **opaque extension reference** — preserved but not interpreted when provider is absent.

This classification drives deletion checks, migration, packaging and diagnostics.

# Unknown/opaque payload contract

Forward-compatible unknown data is stored as a bounded `OpaquePayload` containing namespace/type ID, schema version, declared capabilities and JSON-compatible payload or referenced binary blobs. Unknown payloads:

- round-trip without semantic rewriting where practical;
- are subject to size/depth/resource limits on load;
- cannot execute code or cause arbitrary resource fetches merely by being preserved;
- expose placeholder/read-only behavior when the owning provider is missing;
- are removed only by explicit user action or a migration that declares the transformation.

Unknown **core** schema fields should also be retained when feasible rather than discarded silently.

# Deletion semantics

Deleting an object is a command-level subtree operation with explicit reference analysis. Before commit the mutator computes:

- structural descendants;
- inbound strong/weak references;
- resources whose reference count may reach zero;
- symbol/style/data bindings affected;
- extension-owned references known to registered schemas.

The command then follows a documented policy: cascade structural children, reject unresolved strong references unless the command includes a replacement, clear/fallback weak references, and leave unreferenced resources for explicit/garbage-collection policy. Canonical document mutation never leaves dangling typed references.

# Document validation levels

Define three validation levels:

- `FastInvariantCheck` — debug/test mutation boundary; parent/child symmetry, ID uniqueness, basic reference validity;
- `FullDocumentValidation` — after load/migration and before canonical save when diagnostics demand it;
- `RepairAnalysis` — read-only analysis that proposes repairs for corrupt/partial documents and never mutates automatically.

Validation output is structured `Diagnostic` data with stable diagnostic code, severity, object/resource path, human-facing `TextId` and machine-readable repair candidates.

# Limits and hostile-data posture

The loader enforces configurable hard ceilings before allocating recursively: nesting depth, object count, child count per container, metadata size, string length, resource count and aggregate decoded payload sizes. Limits are high enough for professional documents but explicit, testable and surfaced as structured load diagnostics rather than OOM/panic behavior.

# Serialization boundary

The serialized schema represents semantic IDs and records, not Rust enum layout, pointer topology, `slotmap` keys, hash-map ordering or third-party crate types. Every serialized union uses stable string/type identifiers and versioned payload schemas. This permits internal representation changes without file-format breakage.

# Required tests

In addition to structural validation, V1 tests must cover:

- arbitrary reparent/reorder sequences preserving a valid acyclic tree;
- delete/undo restoring identical persistent IDs and order;
- duplicate producing fresh IDs with correctly remapped internal references;
- save/reload preserving IDs and sibling order;
- runtime-handle rebuild after reload;
- missing provider opaque-payload round-trip;
- malicious depth/count/size inputs failing before excessive allocation;
- randomized object/reference graphs through `proptest` with invariant checks after every accepted mutation.

# Container role clarification

The canonical object graph has **one structural tree**. UI “Layers” are not stored in a parallel layer hierarchy. A group/container may carry a semantic organizational role such as `Layer`; clipping/masking use explicit typed relationships/roles. The exact Rust enum/struct representation is an implementation detail, but serialization and Commands expose one parent/order truth. See 10.5 for behavior and reparent semantics.