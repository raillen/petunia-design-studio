# 09.11.1.1 — PTND Manifest v1 Normative Fields, Limits & Capability Negotiation

# Normative manifest

Required fields:

```
formatId              "ptnd.document"
containerVersion       integer
schemaVersion          integer
minimumReaderVersion   semver/string policy
writer                 object
documentId             DocumentId
requiredCapabilities   string[]
resources              ResourceManifestEntry[]
extensions             ExtensionManifestEntry[]
interchange            InterchangeEntry[]
```

# Writer

appVersion, buildId, platform optional diagnostic, timestamp optional/nonsemantic. Reader must never branch semantics solely on writer product version when schema/capability already expresses requirement.

# Capability

Stable namespaced strings, e.g. ptnd.core.vector-path.v1, [ptnd.color.spot](http://ptnd.color.spot).v1. Required capability means rendering/editing semantics cannot be safely ignored. Optional extension payload must declare optionality.

# Resource entry

ResourceId, kind, path, mediaType, byteSize, sha256, codecVersion, canonical flag, optional metadata. Path normalized before any access.

# Limits

Reader enforces configurable hard limits before allocation:

entry count, total declared uncompressed bytes, single entry bytes, path length, JSON nesting/depth/string length, resource dimensions and compression ratio.

# Integrity

Hash mismatch is corruption diagnostic. Hash is not authenticity. Missing optional derived resource can regenerate; missing canonical resource degrades/fails according object dependency.

# Deterministic writer

Manifest resources sorted by stable deterministic rule (e.g. ResourceId) independent from hash-map iteration. Pretty formatting is optional; semantic canonicalizer used for tests.

# Compatibility

Container version unsupported => refuse before extraction.

Schema newer but minimumReader allows? Use capability/schema compatibility rule, not guessing unknown fields.

Required unknown capability => read-only/compatibility/refuse with explicit issue.

Unknown optional extension => preserve opaque bytes/manifest.

# Tests

Minimal package, maximum bounded counts, unknown optional/required capability, invalid semver, duplicate ResourceId, hash mismatch, duplicate canonical path and deterministic rewrite.