# 21.16 — Effect Presets, Parameter Migration, Plugin Effects & Conformance

# Preset

EffectPreset stores EffectId, schemaVersion, parameter values, optional name/tags/preview. It never stores compiled shader/native pointer.

# Migration

Effect descriptor provides parameter migration vN→vN+1. Missing plugin effect retains opaque canonical node if optional provider absent and displays unavailable state.

# Plugin effect

Third-party provider declares descriptor/schema and execution service. In-process arbitrary pixel callback forbidden. Out-of-process effect is batch/job oriented; high-performance future WASM/native service follows security ADR.

# Shader contribution

Plugins do not inject arbitrary unsandboxed GPU shader into main device in V1. Built-in shaders reviewed/packaged with app.

# Fidelity

Exporter queries effect capability/degradation descriptor. Preset name never influences semantics.

# Conformance

Each built-in effect has CPU oracle, golden images, parameter edge tests and serialization. Plugin effect SDK provides test harness and expected deterministic contract.

# Failure

Unavailable/crashed effect preserves node and renders defined placeholder/pass-through policy with diagnostic; never discards parameters on save.