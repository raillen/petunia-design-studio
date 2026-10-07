# 09.25 — Property & Parameter Schema Registry, Generic Inspectors & Bindable Data Contracts

# PropertyId

Namespaced stable identifier such as ptnd.transform.x, ptnd.stroke.width. Display label/help/unit are metadata.

# Schema

Type, nullable/mixed rules, unit, min/max/step hints, enum values, validation, read-only condition, visibility condition, group/order, control hint, reset/default and automation permissions.

# Sources

Object schemas, tool parameters, effects/adjustments, export options, plugin contributions and preferences can reuse compatible schema primitives.

# Generic inspector

Selection -> common/applicable schemas -> read snapshots -> render controls -> staged edit -> typed Command. Feature-specific custom editor only when generic controls cannot express interaction (curves/pressure graph etc.).

# Multi-selection

Read returns Same(value), Mixed, Unavailable or Error. Set applies semantic command to eligible objects according explicit all-or-compatible policy.

# Binding

Data Merge can bind only properties declaring bindable type/capability. MCP/plugin schema discovery uses same metadata.

# Validation

UI validation is convenience; core command validation authoritative. Error returns PropertyId/path/code/recovery.

# Versioning

Property IDs remain stable; metadata can evolve. Removed property has migration/deprecation mapping.

[09.25.1 — Property Type System, Units, Constraints & Mixed-Value Semantics](09%2025%201%20%E2%80%94%20Property%20Type%20System,%20Units,%20Constraints%203f19bb7d023f81a085a9c469090f0631.md)

[09.25.2 — Canonical PropertyId Catalog by Domain](09%2025%202%20%E2%80%94%20Canonical%20PropertyId%20Catalog%20by%20Domain%203f19bb7d023f819898c7c934b57751f8.md)

[09.25.3 — Property Schema → Qt Inspector → Plugin/MCP Schema Generation Pipeline](09%2025%203%20%E2%80%94%20Property%20Schema%20%E2%86%92%20Qt%20Inspector%20%E2%86%92%20Plugin%20%203f19bb7d023f8111b437d58963b5bf6e.md)