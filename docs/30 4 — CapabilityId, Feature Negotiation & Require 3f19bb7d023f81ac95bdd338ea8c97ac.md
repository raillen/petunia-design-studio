# 30.4 — CapabilityId, Feature Negotiation & Required/Optional Semantics

# Purpose

Describe semantic support independent of app version number.

# CapabilityId

ptnd.capability.<domain>.<feature> with version/range where needed. Examples live_boolean, raster.float16, [color.spot](http://color.spot), text.variable_fonts, extension namespace.

# Required vs optional

PTND manifest lists required capabilities needed for faithful editable interpretation. Optional capability/extension may be preserved/degraded without corrupting core.

# Negotiation

Reader/plugin/MCP handshake compares supported capabilities. Result Full, ReadOnly/Degraded, OpaquePreserve or Unsupported.

# Constraints

Capability can carry limits such as max schema version/bit depth. Do not encode every minor parameter as separate capability if schema version suffices.

# UI

Compatibility warning lists human-readable feature names and affected objects, not just IDs.

# Export

FormatCapabilityDescriptor uses feature IDs aligned where possible but target support grade is separate from application capability.

# Tests

Old reader/new doc, missing plugin extension, optional feature, required unknown and capability version boundary.