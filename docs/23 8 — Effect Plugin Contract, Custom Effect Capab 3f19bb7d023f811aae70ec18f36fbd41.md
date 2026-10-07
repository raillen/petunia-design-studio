# 23.8 — Effect Plugin Contract, Custom Effect Capability & Safety Limits

# Contribution

Plugin can register semantic EffectDescriptor + UI schema + evaluation provider type only if host supports safe runtime path.

# Out-of-process limitation

Third-party Python process cannot receive per-pixel callbacks each frame. Custom effect must:

- map to host-supported shader/expression DSL/WASM compute capability;
- submit offline/background raster transform;
- or expose host composition of approved primitives.

Exact capability is versioned.

# Shader safety

If custom shader path exists, validate language subset, resource limits, no arbitrary buffer access, bounded workgroup/texture size and compilation timeout. Shader cache keyed plugin/version/hash.

# CPU fallback

Plugin effect must declare whether CPU/headless/export is available. If absent, preflight warns/blocks headless/export based policy.

# Missing plugin

Canonical opaque effect preserved; optional baked preview/fallback only if stored and safe. User can disable/bake/remove.

# Permissions

Effect evaluation itself gets document snapshot/resource handles only; network/files require separate grants.

# Tests

Malicious shader/resource request, timeout, plugin disappearance, export without provider and schema version mismatch.