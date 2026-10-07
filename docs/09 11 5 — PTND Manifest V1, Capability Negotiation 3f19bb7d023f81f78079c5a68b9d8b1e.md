# 09.11.5 — PTND Manifest V1, Capability Negotiation, Resource Index, Integrity & Interchange Inventory

# ManifestV1 required

formatId, containerVersion, schemaVersion, minimumReaderVersion, writer, documentId, requiredCapabilities, resources, extensions, interchange, integrity policy.

# Writer

appVersion, buildId, platform optional, schema generator version. Writer metadata is diagnostic, not gate unless known-bad migration policy explicitly exists.

# Capability

CapabilityId namespaced string + minimum semantic version/range when needed. Required capability means reader must implement semantics or refuse/read-only. Optional feature represented by ordinary unknown extension can roundtrip opaquely.

# ResourceRecord

resourceId

kind

path

mediaType

codecId/version

uncompressedSize

storedSize optional

sha256

canonical boolean

derived boolean

externalLink metadata optional

dependencies optional

# Extensions

namespace, schemaVersion, required boolean, entry prefix, provider metadata. Namespace path normalization strict.

# Interchange

Entry records document PDF, per-Surface SVG/PDF/preview assets, generator version, revision fingerprint and fidelity summary. Reader treats them as fallback/preview, not canonical authority.

# Integrity

Per-resource SHA-256 baseline. Manifest/document hashes may protect index consistency. Package signing is separate trust layer and cannot be confused with corruption hash.

# Capability open behavior

All required supported -> normal.

Unsupported required but interchange available -> offer read-only/interchange view.

Unsupported required no safe fallback -> refuse with list.

Unknown optional -> preserve opaque entries if package rewritten.

# Tests

Capability negotiation table, hash mismatch, duplicate resource path, extension namespace conflict, read-only fallback and manifest migration.