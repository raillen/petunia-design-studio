# 09.3.3 — Command Schema, ChangeSet Semantics, Conflict Rules & Replay Guarantees

# Command envelope

Each command has CommandTypeId, schemaVersion, typed payload, target IDs, optional expected_revision, provenance source, and validation metadata.

# Command categories

Create, Delete, PropertyMutation, Transform, Hierarchy, GeometryTopology, RasterDelta, TextEdit, Resource, EffectGraph, Surface/Layout, DataMergeConfig.

# Validation

Two-stage:

1. structural/type validation;
2. semantic validation against current DocumentSnapshot.

No Command may depend on QWidget state or pointer addresses.

# ChangeSet

```
ChangeSet
  new_revision
  created_ids
  deleted_ids
  modified_properties {ObjectId -> PropertyId[]}
  hierarchy_changes
  geometry_invalidations
  raster_regions
  resource_changes
  text_story_changes
  surface_changes
```

ChangeSet is sufficient for UI models and derived invalidation without diffing entire document.

# Conflict rules

expected_revision mismatch returns StaleRevision unless command type has explicit rebase strategy. Generic "best effort" mutation is forbidden.

# Replay

Commands used for recovery/replay must be deterministic. External file/network reads occur before command creation and become Resource/StagedResult references with fingerprints.

# Provenance

source = ui|shortcut|plugin:<id>|mcp:<client>|migration|recovery. Provenance is diagnostic/audit metadata and never affects semantics.

# Versioning

Old command records in recovery journal migrate stepwise or recovery loader materializes canonical snapshot. Public plugin/MCP Action schemas need not expose raw internal command schema.

# Tests

Command serialization where applicable, deterministic replay, invalid target, deleted ID, stale revision, migration, ChangeSet minimality and UI model update parity.