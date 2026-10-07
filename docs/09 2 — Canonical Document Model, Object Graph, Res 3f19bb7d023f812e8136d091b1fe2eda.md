# 09.2 — Canonical Document Model, Object Graph, Resources & Stable IDs

# Document root

DocumentId, schema metadata, color context, Surface list, resources, styles, symbols, data bindings, extension namespaces.

# Stable IDs

128-bit UUID-like IDs or equivalent strongly typed wrappers: ObjectId, LayerId, SurfaceId, ResourceId, StyleId, SymbolId. C++ uses distinct wrapper types to avoid mixing IDs. Python exposes frozen typed handles/UUID wrappers.

# Hierarchy

Surface contains/render-selects root content according to model; content graph uses parent/child ownership with cycle prevention. Layer tree order is semantic paint order.

# Object families

Group, VectorPath, ParametricShape, TextObject, RasterLayer, AdjustmentLayer, LiveFilter, Mask, SymbolInstance, PlacedResource, Compound/Boolean object.

# Resources

Images, fonts, ICC profiles, brush resources, patterns, data sources and external links are indexed by ResourceId. Path is metadata/location, never identity.

# Appearance

Objects reference typed AppearanceStack: fills, strokes, opacity/blend, effects and masks. Each entry has stable local identifier for UI editing.

# Extension payload

Optional plugin namespace data is isolated by namespace/version. Unknown optional payload round-trips opaquely; unknown required semantics triggers compatibility handling.

# Validation

No dangling mandatory IDs, hierarchy cycles, NaN/Inf geometry, invalid transforms, missing required resource metadata or impossible color states. Validation runs after load/migration and before save commit.