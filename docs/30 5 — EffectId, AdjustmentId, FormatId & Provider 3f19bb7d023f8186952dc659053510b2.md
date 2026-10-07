# 30.5 — EffectId, AdjustmentId, FormatId & Provider Registry Contracts

# Effect/Adjustment

ID + parameter schema version + CPU/GPU capability + ROI descriptor + UI editor type + serialization/export feature IDs.

# FormatId

ptnd.format.ptnd/png/jpeg/tiff/svg/pdf etc plus profile/version. Importer/Exporter descriptors register directions, sniff signatures, options schema and capability matrix.

# Provider

Third-party IDs namespace-own effect/data/export provider contributions. Provider unload leaves canonical unknown optional nodes only if storage/extension contract permits; otherwise installation must declare required capability.

# Selection

Application chooses importer by explicit format/signature/sniff ranking, not registry insertion order.

# Versioning

Adapter implementation version can change without FormatId if external semantics stay; profile/schema capability versions record meaningful behavior.

# Tests

Duplicate provider, sniff ambiguity, unsupported effect provider and exported capability descriptor consistency.