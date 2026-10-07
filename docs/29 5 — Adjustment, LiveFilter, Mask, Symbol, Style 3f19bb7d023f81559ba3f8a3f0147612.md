# 29.5 — Adjustment, LiveFilter, Mask, Symbol, Style & Resource Schemas

# Adjustment

Common Object/Effect node with AdjustmentId, schemaVersion, typed params, scope/order, opacity/blend and masks.

# LiveFilter

EffectId/schemaVersion, params, quality-independent canonical values, enabled, mask, input scope. Render backend data is derived.

# Mask

Tagged VectorMask, PixelMask or compound mask structure with invert/combine semantics and target relationship.

# SymbolDefinition

SymbolId, root object IDs/document fragment ownership, exposed override PropertyIds and resource dependencies.

# SymbolInstance

Object + SymbolId, transform and typed override map keyed by stable semantic property path/ID. Detach materializes local copy with ID remap.

# Style

Object/Character/Paragraph style records with parent/reference rules and typed property subsets.

# Resource

ResourceId, kind, provenance, embedded/linked state, media/schema, fingerprint/hash, file/grant metadata, profile metadata and extension fields. Path is location, not identity.

# Validation

References typed, cycles in style/symbol definition graphs rejected where invalid, missing optional linked resources become explicit Missing state rather than schema deletion.