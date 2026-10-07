# 09.11 — .PTND Native File Format, Serialization, Migrations, Autosave & Recovery

# Identity

Canonical suffix: **.PTND**.

Canonical formatId: **ptnd.document**.

The package/schema identity never depends solely on file extension.

# Container

ZIP-compatible constrained profile. Required minimum:

```
mimetype
manifest.json
document/document.json
```

Recommended layout:

```
resources/images/<resource-id>.<ext>
resources/raster/<layer-id>/<level>/<x>_<y>.ptile
resources/fonts/<resource-id>/
resources/icc/<resource-id>.icc
resources/data/<resource-id>/
extensions/<namespace>/
interchange/document.pdf
interchange/surfaces/<surface-id>.svg
previews/thumbnail.webp
cache/
```

interchange/previews/cache são derivados/optional; cache é descartável.

# ZIP security profile

UTF-8 forward-slash paths; no absolute path, drive prefix, traversal, symlink extraction semantics, duplicate canonical paths, encryption/multidisk V1. Limits para entries, compressed/uncompressed sizes e compression ratio.

# Manifest

containerVersion, schemaVersion, minimumReaderVersion, writer version/build, DocumentId, requiredCapabilities, resource index, extension namespaces, interchange inventory, integrity data.

# Document JSON

Versioned JSON Schema; stable namespaced type strings; semantic arrays explicit; non-finite numbers rejected. Large binaries nunca base64 inline.

# Resource index

ResourceId, type, entry path, media type, byte size, codec/schema version, SHA-256 baseline, canonical/derived/external flags.

# Version dimensions

container, core schema, resource codec, extension namespace e interchange generator version independentes.

# Load

bounded envelope parse -> manifest validate -> capability check -> resource index validate -> schema migration stepwise -> canonical validation -> open session. Unknown optional extension preserved; unsupported required semantics => read-only/compatibility/refuse with diagnostic.

# Atomic save

Create temp beside destination when possible -> serialize snapshot -> fsync file/data as practical -> reopen/validate package -> atomic replace -> fsync directory where supported. Nunca progressive overwrite do único good copy.

# Autosave

Separate recovery journal/checkpoints, keyed by document/session identity and revision. Autosave failure does not clear dirty state.

# Recovery

Candidate opens as normal **unsaved recovered document**. Original is never modified during recovery/salvage.

# Corruption

Integrity hashes, bounded salvage, structured report. Repair writes new file only.

# Migrations

Pure/deterministic, one version step each, golden fixtures, idempotence where relevant, no GUI/network dependencies.

[09.11.1 — .PTND V1 Package Layout, Manifest & Canonical JSON Schema](09%2011%201%20%E2%80%94%20PTND%20V1%20Package%20Layout,%20Manifest%20&%20Canon%203f19bb7d023f8106b4fde3e0c317e02c.md)

[09.11.2 — Raster Tile Resource Codec, Compression, Deduplication & Large-Document Storage](09%2011%202%20%E2%80%94%20Raster%20Tile%20Resource%20Codec,%20Compression,%203f19bb7d023f81b89e16cdbaee3a0e59.md)

[09.11.3 — Autosave Journal, Checkpoints, Crash Recovery, Atomic Replace & Salvage](09%2011%203%20%E2%80%94%20Autosave%20Journal,%20Checkpoints,%20Crash%20Rec%203f19bb7d023f8177b380c574c854d0a0.md)

[09.11.4 — PTND Core Schema V1: Document, Surface, Object Union, Appearance & Resources](09%2011%204%20%E2%80%94%20PTND%20Core%20Schema%20V1%20Document,%20Surface,%20O%203f19bb7d023f814b9005e9b97eb41e84.md)

[09.11.5 — PTND Manifest V1, Capability Negotiation, Resource Index, Integrity & Interchange Inventory](09%2011%205%20%E2%80%94%20PTND%20Manifest%20V1,%20Capability%20Negotiation%203f19bb7d023f81f78079c5a68b9d8b1e.md)

[09.11.6 — PTND Object Schema Examples: Vector, Raster, Text, Masks, Effects, Symbols & Data Merge](09%2011%206%20%E2%80%94%20PTND%20Object%20Schema%20Examples%20Vector,%20Rast%203f19bb7d023f81c2ad6fcc1545682d73.md)

[09.11.7 — PTILE Binary Codec V1 Candidate: Header, Payload, Compression, Checksums & Evolution](09%2011%207%20%E2%80%94%20PTILE%20Binary%20Codec%20V1%20Candidate%20Header,%20%203f19bb7d023f81a4a7dde7e9db336855.md)

[09.11.8 — PTND Migration Framework, Forward Compatibility, Opaque Preservation & Reference Validator CLI](09%2011%208%20%E2%80%94%20PTND%20Migration%20Framework,%20Forward%20Compat%203f19bb7d023f8154ad32c9de1320c0f4.md)