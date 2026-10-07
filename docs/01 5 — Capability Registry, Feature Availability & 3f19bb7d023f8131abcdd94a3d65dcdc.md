# 01.5 — Capability Registry, Feature Availability & Missing-Capability Degradation

# Capability

Stable namespaced identifier for product/runtime/document functionality. Examples geometry.boolean.v1, [color.spot](http://color.spot).v1, exporter.pdf.v1.

# Registry

Built-in modules/plugins register descriptors with version, dependencies, contributions and runtime availability.

# Document

PTND declares required capabilities only when semantics cannot safely degrade. Optional features/extensions can preserve opaque payload/fallback.

# UI

ActionAvailability consults object capabilities, active module, permissions and backend. Disabled reason identifies missing capability rather than hiding feature.

# Plugin

Handshake negotiates semantic SDK capabilities independent from app version string.

# Export/import

Adapter capability matrices use feature IDs compatible with registry taxonomy where practical.

# Missing

Open document can enter read-only/degraded mode if required capability missing and safe display/interchange fallback exists; otherwise refuse with precise report.

# Tests

Plugin removed, backend feature unavailable, newer PTND optional extension and required capability negotiation.