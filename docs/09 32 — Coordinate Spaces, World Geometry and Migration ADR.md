# 09.32 — Coordinate Spaces, World Geometry and Migration ADR

<aside>

**Status:** `ACCEPTED_V1` for the coordinate/evaluation contract; implementation is executing in `P07-G05`. The domain/world-geometry slice and a toolkit-neutral Freya snapshot are in place; the complete vector/text renderer and release performance baseline remain open.

**Scope status:** `V1_REQUIRED` foundation for vector, hierarchy, selection, canvas, export and performance evidence.

**Supersedes:** nothing. The current implementation had an ambiguous `to_path()`/placement convention that this ADR makes explicit before migration.

</aside>

# Decision question

How should Aubrieta represent object geometry, placement, hierarchy, viewport projection and cached derived data so that a single canonical truth is visible to tools, hit-testing, culling, rendering, export and automation?

# Context

The current v1 model stores object `bounds`, `rotation`, `shape`, `parent` and `children`. Parametric shape factories currently create paths whose coordinates include the bounds origin, while `local_transform()` is documented as `T(bounds_origin) * R(rotation)`. `world_transform()` composes those transforms through the parent chain. Consumers independently read `bounds`, `to_path()`, `evaluated_path()` and rotation.

This permits three correctness failures:

- applying a world transform to a path that already contains the placement origin, or applying placement a second time;
- placing a child at a local bounds value as if it were a world value;
- using an axis-aligned base bounds box for culling, hit-testing and selection after rotation or hierarchy composition.

The Freya canvas currently has a private snapshot based on stored bounds and shape. It does not compose hierarchy/world geometry. A performance benchmark on that path would measure a known-incorrect projection.

# Constraints

- `AGENTS.md` requires domain/application code to remain toolkit-independent and all mutations to use `Action → Command → DocumentMutator → ChangeSet`.
- `09.4` requires local geometry, world placement, invalidation, cache generations and quality contexts to be distinguishable.
- `09.5` requires finite f64 coordinates, named tolerance policies and no silent geometry normalization.
- `09.10`, `09.10.1` and `14.6` require versioned native payloads, staging migrations, atomic saves and no in-place interpretation of old files.
- `10.1`, `10.5`, `10.7` and `08.23` require explicit coordinate spaces, revision-safe previews, one gesture/one undo and accessible semantic interaction.
- `14.1`, `14.2`, `14.3` and `14.8` require deterministic fixtures, structural evidence, release metadata and reproducible commands.
- The current native schema is `1`. Existing documents must remain loadable; unknown or ambiguous historical meaning must not be guessed.

# Options

## Option A — Change `to_path()` and `evaluated_path()` immediately to local coordinates

This is the clean final model, but changing the meaning of existing methods while consumers still assume world coordinates would be a silent data migration. It also mixes native-format migration, tool migration, renderer migration and cache migration.

**Rejected as the first implementation step.** It may become the destination after the explicit compatibility API and migration edge are in place.

## Option B — Add explicit local/world APIs and keep a bounded legacy compatibility path (chosen)

Add namespaced APIs whose coordinate space is visible in the name and whose result is derived through one resolver. Migrate creation, tools, cache, selection, export and canvas incrementally. Keep existing v1 readers only where the source frame is known or a migration diagnostic is explicit.

**Chosen.** It prevents a silent semantic break while making the correct model executable and testable.

## Option C — Keep the current implicit convention and fix only Freya

This is the smallest diff but would preserve divergent math in the core, exporters and tools. It would make the adapter responsible for domain truth and leave the performance problem in the wrong layer.

**Rejected.** It violates the stable semantic bridge and the evaluation contract.

# Decision

## Coordinate spaces

1. **Local geometry space** contains the editable path or parametric geometry before placement. It does not include `bounds` or `rotation`.
2. **Parent/placement space** contains the object frame origin (`bounds.x/y`) and local transform relative to its parent. For the current TRS model, placement is `T(bounds_origin) * R(rotation)`.
3. **World/pasteboard space** is the composition of parent placements: `W_parent * (T(bounds_origin) * R(rotation))`.
4. **Screen space** is produced by `ViewportCamera`; screen values are presentation and interaction coordinates, never persisted vector truth.

## Canonical APIs

The document/application boundary must expose explicit, toolkit-neutral operations equivalent to:

- `base_path_local(id or object)`;
- `evaluated_path_local(id or object)`;
- `world_transform(id)`;
- `base_path_world(id)`;
- `evaluated_path_world(id)`;
- `evaluated_bounds_local(id)`;
- `evaluated_bounds_world(id)`;
- `world_aabb(id, padding policy)`.

Names may be finalized to match repository naming, but the coordinate space and direction must be explicit. A world result must not be passed to an API that applies placement again.

## Bounds and rotation

`bounds` is a nominal placement frame, not a universal AABB. Local evaluated bounds and world AABB are separate derived values. A rotated object's world AABB includes the composed transform and any documented render padding for stroke/effects. Culling, selection and handles consume the declared world projection policy rather than silently using stored base bounds.

## Hierarchy

A root object may be placed directly in the Surface/pasteboard frame. A child placement is relative to its parent. Group/ungroup/reparent commands preserve `W_before ≈ W_after` unless the user explicitly requests a local-coordinate conversion. Container geometry is derived from eligible descendants and containers are not painted unless their role/paint contract explicitly requires it.

Cycles, dangling parents, non-finite transforms and non-invertible matrices are rejected with structured diagnostics. Scale/shear reparent operations are rejected until the persisted model can represent the complete affine safely.

## Modifiers

Geometry modifiers are evaluated in the object's local frame by default. Their serialized parameters must declare their space:

- Contour offset: local distance;
- Perspective quad: local source/destination frame;
- Crop rectangle: local frame;
- Transparency gradient: object-local vector unless an explicit document-anchored variant is introduced.

A document-anchored variant, if needed, is a separately typed and versioned semantic contract. It is not inferred from an existing field.

## Migration

The current schema remains readable. A later schema migration, when required, must:

1. parse the old payload into staging state;
2. classify path, modifier and hierarchy frame ambiguity;
3. convert only when the mapping is deterministic;
4. emit migration diagnostics and a digest/summary;
5. preserve unknown forward data according to the compatibility policy;
6. validate the canonical result;
7. write the new schema only through the normal atomic Save path.

No migration may subtract `group.bounds` from a path or rotate a world path by a second time. Ambiguous cases fail preflight or require an explicit user repair policy.

## Canvas and interaction

The shell/application owns one toolkit-neutral scene snapshot. It contains the document revision, camera, painter order, stable `ObjectId`, world projection data, style data, visibility/selection/hover state and coordinate-space-aware overlays. Freya renders that snapshot; it does not own a second placement or culling algorithm.

Normalized pointer input is converted once from screen to document. Select previews capture the document revision, selected IDs, world geometry, pivot/frame and coordinate space. Preview, commit and cancel share the same resolver. Stale commits are rejected or safely rebased only where the operation contract explicitly permits it.

# Consequences

- The domain becomes the owner of geometry truth; Freya and future shells become replaceable presentation adapters.
- Paths, modifiers, hierarchy, selection, culling, hit-testing and exports share a testable world-space oracle.
- The first migration is additive and compatibility-preserving; it may temporarily retain deprecated legacy readers.
- The world snapshot and cache can be benchmarked independently from Freya painting and the CPU reference compositor.
- Some existing consumers must migrate deliberately; callers cannot rely on a compiler error because the old method's coordinate meaning changes only after a migration.
- Visual parity is not delivered by this ADR. The Freya renderer still requires a complete vector/text/appearance backend and pixel evidence.

# Compatibility classification

- **Architecture contract:** `SchemaMigrationRequired` for a persisted local-path/modifier interpretation; `BehaviorCompatibleButObservable` for derived world projection.
- **Public internal API:** additive APIs first; deprecate legacy path readers before removal.
- **Native format:** version 1 remains readable; any new interpretation is a new schema edge with migration and release notes.
- **UI:** the visible canvas projection may change, but semantic Action/Command/undo/selection behavior must remain stable or be covered by a migration diagnostic.
- **Plugin/MCP:** no permission change in this gate; future public frame queries are versioned and report their coordinate space.

# Required evidence

- root, child, rotated and nested-group fixtures;
- finite/invalid transform cases;
- `W_before/W_after` group/ungroup/reparent tests;
- no-double-transformation differential tests;
- local/world path and AABB oracle;
- hit-test/culling/selection/handle/preview/export agreement;
- save/reopen and migration preflight fixtures;
- deterministic structural snapshot and release benchmark metadata;
- EN + pt-BR documentation and ADR registry consistency.

# Revisit trigger

Revisit this ADR only if a future persisted affine model (non-uniform scale/shear), a document-anchored modifier, multi-surface coordinate systems, or measured renderer requirements demonstrate that the chosen local/world contract cannot represent a required V1 workflow. Any superseding decision must identify migration, cache invalidation, export and UI compatibility explicitly.

# Rollout order

1. Add explicit local/world resolver APIs and unit/property tests without changing legacy callers.
2. Migrate hierarchy and modifier operations in bounded command groups.
3. Migrate cache/spatial index, selection, hit-testing and Select preview.
4. Extract the toolkit-neutral scene snapshot and connect Freya to it.
5. Run deterministic fixtures, structural goldens and release benchmarks.
6. Only then consider a persisted migration and a full vector renderer.
