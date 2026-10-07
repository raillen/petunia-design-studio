# 25.6 — Capability Matrix Schema, Adapter Declaration & Automated Conformance

# Machine-readable matrix

Each adapter ships FormatCapabilityDescriptor:

formatId/profile/version; directions; feature capability map; constraints; degradation strategies; color/text/page limits; security notes.

# Feature IDs

Examples: geometry.cubic_path, appearance.multiple_fills, stroke.variable_width, text.frame_flow, [color.spot](http://color.spot), document.multiple_surfaces, effect.gaussian_blur, mask.vector.

# Constraint

Capability can be Native, NativeWithConstraints, Expand, Outline, Rasterize, Approximate, Unsupported. Constraint object includes conditions such as max bit depth or profile version.

# Preflight integration

Export analyzer compares evaluated document feature inventory against descriptor to create deterministic degradation plan.

# Import report

Parser/converter records encountered source capabilities and conversion grade per class/object when relevant.

# CI

Generated fixtures exercise every claimed Native/Exact row. Missing conformance fixture blocks upgrading a capability claim.

# Documentation

User-facing support table generated from same descriptor, preventing marketing/docs drift.