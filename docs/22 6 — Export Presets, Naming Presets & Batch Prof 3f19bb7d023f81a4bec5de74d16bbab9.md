# 22.6 — Export Presets, Naming Presets & Batch Profiles

# ExportPreset

PresetId, ExporterId/version range, schemaVersion, format options, target profile/color settings, fidelity policy, metadata policy, area defaults and post-export actions limited to safe built-ins.

# NamingPreset

Tokenized safe filename template + sanitization/collision policy. Data Merge tokens capability declared.

# Scale variants

List of scale or target dimensions, suffix/prefix and resampling override. One logical export target can generate variants.

# Compatibility

When exporter updates schema, preset migrates or shows incompatible state. Missing plugin exporter does not delete preset.

# Scope

Built-in immutable, user, workspace/project and document-embedded optional. Sensitive destination grants are not stored in portable preset.

# Import/export

Preset pack excludes filesystem grants/secrets. Conflicting IDs use replace/duplicate.

# Tests

Exporter missing/version mismatch, migration, filename tokens, color profile dependency and batch variants.