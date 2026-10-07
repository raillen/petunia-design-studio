# 29.6 — DataSource, Binding, ExportTarget, ExtensionPayload & Metadata Schemas

# DataSource

DataSourceId, providerId/version, configuration excluding secrets, schema fingerprint, field descriptors, resource/file grant reference and refresh metadata.

# Binding

BindingId, sourceId, target ObjectId + PropertyId/path, Expression AST/source version, fallback/null/format policy and expected type.

# ExportTarget

ExportTargetId, area kind/object/Surface/slice geometry, name, ExportPresetId/inline override, scales/suffixes, enabled and destination grant/path template reference.

# ExtensionPayload

Namespace/provider, schemaVersion, required/optional semantic flag, opaque canonical payload/resource refs. Unknown optional preserved; required unknown blocks full editable mode.

# Metadata

Namespaced metadata dictionary only for truly extensible descriptive metadata. Core product semantics must use typed fields, never hidden generic metadata.

# Secrets

Credentials/tokens never canonical PTND DataSource/Resource fields. Store credential reference to OS/app secure store if provider requires.

# Validation

Binding target property type, provider availability state, safe expression, export filename template and extension capability negotiation.