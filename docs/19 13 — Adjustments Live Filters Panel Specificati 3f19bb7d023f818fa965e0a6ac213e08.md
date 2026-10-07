# 19.13 — Adjustments / Live Filters Panel Specification

# Catalog

Searchable adjustment/filter categories with favorites/recent optional. Add action creates live node in current hierarchy/target.

# Selected effect

Parameter editor generated from EffectDescriptor/PropertySchema, with specialized Curves/graph control where needed.

# Preview

Scrub stages EffectPreviewSession; one commit. Enable/disable, mask add, duplicate, remove, reorder actions.

# Target

Panel shows where effect applies: clipped to layer, group, adjustment layer position.

# CPU/GPU status

Do not expose implementation detail normally; diagnostics can show backend. If effect unavailable on current backend, host falls back or explains limitation.

# Tests

Add/undo, preview cancel, effect reorder, mask, missing plugin effect and schema migration.