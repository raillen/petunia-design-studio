# 09.11.1 — .PTND V1 Package Layout, Manifest & Canonical JSON Schema

# Required entries

```
mimetype
manifest.json
document/document.json
```

mimetype contains the registered/project media type once finalized and should be stored uncompressed first when writer can guarantee.

# Canonical vs derived

Canonical: manifest, document JSON, canonical resources.

Derived: previews, interchange PDF/SVG, caches.

Reader may delete/regenerate derived entries without semantic loss.

# Manifest example

```json
{
  "formatId": "ptnd.document",
  "containerVersion": 1,
  "schemaVersion": 1,
  "minimumReaderVersion": "1.0",
  "writer": {"appVersion": "1.0.0", "buildId": "..."},
  "documentId": "...",
  "requiredCapabilities": [],
  "resources": [],
  "extensionNamespaces": [],
  "interchange": []
}
```

# Document JSON

Stores document metadata, color context, Surfaces, object hierarchy, styles/symbols, resource references, Data Merge bindings and extension references. Binary pixel/font/profile data lives outside.

# IDs

Serialized as canonical strings. Object ordering uses arrays. Maps cannot rely on implementation hash iteration ordering for semantic order.

# Numbers

Finite JSON only. Coordinates/parameters have documented units/ranges. Parser rejects overflow/absurd nesting before allocation amplification.

# Extensions

extensions/provider.namespace/... with manifest declaration version + required/optional. Unknown optional preserved byte-for-byte where safe.

# Schemas

Publish JSON Schema for manifest/document and each versioned extension contract. Reference validator CLI consumes same schemas plus semantic checks.

# Fingerprints

SHA-256 baseline per canonical resource. Package-level signature is separate future trust feature; hash alone is corruption detection.

[09.11.1.1 — PTND Manifest v1 Normative Fields, Limits & Capability Negotiation](09%2011%201%201%20%E2%80%94%20PTND%20Manifest%20v1%20Normative%20Fields,%20Lim%203f19bb7d023f817aad0cee7c153d4176.md)

[09.11.1.2 — document.json v1 Core Schema: Document, Surface, Object Envelope & Hierarchy](09%2011%201%202%20%E2%80%94%20document%20json%20v1%20Core%20Schema%20Document,%203f19bb7d023f816c9ec5d2fec997c3c7.md)

[09.11.1.3 — PTND v1 Vector, Shape, Appearance, Text & Effect Object Schemas](09%2011%201%203%20%E2%80%94%20PTND%20v1%20Vector,%20Shape,%20Appearance,%20Tex%203f19bb7d023f811d8558cc4794627701.md)

[09.11.1.4 — PTND v1 Raster, Mask, Resource, Adjustment & Data Merge Schemas](09%2011%201%204%20%E2%80%94%20PTND%20v1%20Raster,%20Mask,%20Resource,%20Adjust%203f19bb7d023f81b1b002f4055c70f044.md)

[09.11.1.5 — PTND JSON Schema Generation, Canonicalization, Migrations & Reference Validator CLI](09%2011%201%205%20%E2%80%94%20PTND%20JSON%20Schema%20Generation,%20Canonical%203f19bb7d023f814eaa1fd49ad6099641.md)