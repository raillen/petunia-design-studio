# 09.11.1.2 — document.json v1 Core Schema: Document, Surface, Object Envelope & Hierarchy

# Root

```json
{
  "schema":"ptnd.document.v1",
  "id":"...",
  "name":"...",
  "units":"px",
  "colorContext": {...},
  "surfaces": [...],
  "objects": {...},
  "rootOrder": [...],
  "resources": [...],
  "styles": {...},
  "symbols": {...},
  "dataSources": {...},
  "extensions": {...}
}
```

Exact JSON Schema lives in repository and this page is normative prose companion.

# Object envelope

Every canonical object includes:

id, type, schemaVersion/typeVersion, name optional, visible, locked, parent/container relationship, transform, opacity, blendMode, appearance reference/inline stack, masks/effects references as designed, extension data.

# Hierarchy

Order is explicit array of ObjectIds under container/Surface; an object appears exactly once in ownership hierarchy except referenced definitions/resources. Parent fields and child arrays must agree or validator rejects/repairs only in salvage mode.

# Surface

SurfaceId, role, rect/size/position, background, bleed, margins, columns, baseline/grid refs, export flags, template ref, root child order.

# Coordinates

All numeric canonical coordinates finite double JSON numbers in document units; rotation stored radians or degrees chosen once and schema states it. Transforms stored fixed-length affine matrix with semantic decomposition optional.

# Unknown type

Unknown core object type is required capability failure. Unknown plugin extension object may preserve opaque payload if optional and has fallback bounds/preview rules only when extension contract permits.

# Validation invariants

Unique IDs globally per domain; no parent cycles; referenced IDs exist or are explicitly external; order arrays no duplicates; transform finite; opacity 0..1; required type fields valid.

# Salvage

Normal reader never silently invents hierarchy. Salvage mode can detach invalid objects to Recovery group and records issue report.

# Tests

Deep nesting limit, cycle, duplicate object, dangling child, multi-parent, unknown type, invalid transform, deterministic ordering.