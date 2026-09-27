# 09.10 — Native File Format, Serialization, Versioning, Migrations, Autosave & Recovery

# Amendment 2026-09-21 — Native Format Identity

O formato nativo passa a usar **.PTND** como sufixo canônico gravado pelo aplicativo. .aubrieta/.aubri tornam-se legacy import aliases somente para migração, se/onde fixtures históricas existirem.

O schema interno deve receber nova identidade/versionamento sem depender apenas da extensão. A migração precisa preservar documentos existentes e ter round-trip/corruption/atomic-save evidence antes de remover leitores antigos.

# Format goals

Human-auditable manifest, robust recovery, forward-compatible opaque extension data, incremental resource replacement and deterministic migrations.

# Container

Versioned **`.aubrieta`** ZIP-compatible container with **`.aubri` as an accepted short filename alias for the exact same package/schema**, plus `manifest.json`, canonical JSON structural payload, resources, embedded ICC/fonts/images/data, previews, derived interoperability representations and optional disposable caches. `.petunia` remains reserved for Petunia3D; `.pds` and `.abrt` are rejected. The package layout and schemas are public and third-party implementable. Canonical internal format identity is `org.aubrieta.design.document`, independent of suffix.

`document/document.json` is the **canonical V1 structural payload**. Large images, raster tiles, fonts, ICC profiles and other binary resources remain separate indexed entries rather than base64 inside JSON. Container compression handles structural JSON verbosity; optional binary acceleration caches may exist but are never canonical.

# Manifest

Application format version, minimum reader, document UUID, created/modified metadata, resource index/fingerprints, required capability namespaces, optional extension payload inventory and integrity metadata.

# Atomic save

Write new container/temp path → fsync where practical → validate → atomic replace. Never overwrite sole good copy progressively.

# Versioning

Separate container version, document schema version and extension namespace versions. Load pipeline: parse envelope → validate → migrate stepwise → validate canonical model. No giant version switch.

# Migrations

Pure/deterministic where possible, individually tested with golden fixtures. Migrations cannot depend on GUI or network. Unknown optional extension data round-trips when safe.

# Autosave

Autosave is a recovery journal/checkpoint distinct from explicit Save. Use dirty revision and incremental/resource-aware strategy. Autosave failures never clear document dirty state.

# Crash recovery

On startup identify last explicit save and recovery revision/time. Opening recovery creates a normal unsaved document until user chooses Save; discarding requires explicit identity confirmation.

# Corruption handling

Container/resource checksums where useful; salvage unaffected resources; produce structured diagnostics and never mutate original corrupt file during repair attempt.

# Interoperability profile

Normal explicit saves can maintain derived `interchange/document.pdf` plus editable SVG projections per Surface. These are compatibility views, never the document truth. Aubrieta also exposes direct SVG/PDF save/export modes and an optional compatibility-companion workflow for external collaboration.

See child page **09.10.1 — Open Native Format, Interoperability Profile & Compatibility Representations** for the canonical package layout, open-spec rules and external-editor strategy.

# Tests

Every historical schema fixture, interrupted writes, disk-full, permission failure, corrupt zip/manifest/resource, migration idempotence where relevant, forward-unknown extension roundtrip, compatibility projection generation and third-party fixture readability.

[09.10.1 — Open Native Format, Interoperability Profile & Compatibility Representations](09%2010%201%20%E2%80%94%20Open%20Native%20Format,%20Interoperability%20Pro%203db9bb7d023f81a4a6c5d9edfcc5b904.md)

# Package conformance contract

The V1 package is a ZIP container, but **ZIP behavior is constrained by Aubrieta's own conformance profile**. Readers/writers must not rely on arbitrary ZIP features merely because a library supports them.

Required V1 package rules:

- UTF-8 normalized forward-slash entry names;
- no absolute paths, drive prefixes, `..` traversal or symlink extraction semantics;
- duplicate canonical entry paths are invalid;
- case-sensitive internal paths regardless of host filesystem;
- bounded entry count, compressed size, declared/uncompressed size and compression ratio;
- encryption is not part of V1 native format;
- multi-disk ZIP is unsupported;
- `mimetype` should be a small stored/uncompressed first entry when the writer can guarantee it;
- unknown noncanonical entries may be preserved only under approved extension namespaces or ignored according to spec; they never execute.

The specification publishes exact required/optional entry paths and their media types.

# Canonical package layout — V1 minimum

A conforming editable document contains at least:

```
mimetype
manifest.json
document/document.json
```

Resources referenced by canonical payload must be present or explicitly declared external/missing according to resource policy. `interchange/`, `previews/` and `cache/` are optional derived sections.

# Manifest implementation schema

`manifest.json` must contain stable machine-readable fields rather than free-form application metadata. Minimum conceptual fields:

```
formatId: "org.aubrieta.design.document"
containerVersion
schemaVersion
minimumReaderVersion
writer { appVersion, buildId }
documentId
resourceIndex[]
requiredCapabilities[]
extensionNamespaces[]
interchangeRepresentations[]
integrity metadata
```

Creation/modification timestamps are metadata only and never used for migration/canonical ordering or cache validity.

# Resource index

Every packaged binary/canonical resource entry is indexed by stable resource ID/type, path, byte length and content fingerprint/hash. Where meaningful it also records media type, codec, schema/codec version and whether the resource is canonical, external reference, derived preview or cache.

A resource path is not its identity. Internal paths may be reorganized by a future container migration while ResourceId references remain stable.

# Integrity policy

Use modern cryptographic content hashes (canonical recommendation: SHA-256 baseline for interoperable V1 integrity metadata) for resources where integrity/salvage/deduplication benefits justify it. Hashes detect corruption; they are not digital signatures/trust proof.

Manifest/package signature support is a separate security/distribution concern and must not be improvised into V1 document semantics.

# JSON schema policy

`document/document.json`, manifest and extension payload schemas are published as versioned JSON Schemas. Serialized unions use stable namespaced string type IDs rather than Rust enum discriminants.

Writer rules:

- emit canonical field names/types for the target schema version;
- omit only fields whose schema-defined defaults preserve semantics;
- do not serialize HashMap iteration order as semantic order;
- preserve semantic sibling order explicitly in arrays;
- reject non-finite JSON numbers at serialization boundary;
- use deterministic enough formatting/order in conformance fixtures to make diffs useful, without claiming byte-for-byte package determinism when timestamps/compression differ.

# Version dimensions

Keep independent:

1. **containerVersion** — ZIP/package layout rules;
2. **schemaVersion** — core canonical document JSON semantics;
3. **resource codec/schema versions** — raster chunks/other typed payloads;
4. **extension namespace version** — provider-owned opaque/known extension data;
5. **interchange generator version** — derived PDF/SVG representation metadata.

A bump in one dimension does not force arbitrary bumps in all others.

# Reader compatibility policy

The manifest declares minimum reader capability/version. Reader behavior:

- unsupported newer **required** core/capability semantics → open read-only/compatibility mode or refuse with actionable diagnostic according to affected scope;
- unknown **optional** extension namespace → preserve opaque payload and continue;
- unsupported derived cache/interchange entry → ignore/regenerate;
- missing required canonical resource → open with explicit degraded/missing-resource state where salvageable, never silently replace.

# Migration pipeline

Load uses immutable source package → parsed old schema model → ordered migration chain → canonical current model → full validation. Migration functions never mutate the source file/package in place.

Each migration declares:

- from/to schema version;
- prerequisites;
- data transformed;
- unknown-data policy;
- diagnostics/warnings;
- whether reverse/down-save is possible (normally no unless explicitly implemented);
- golden fixtures.

Skipped migration versions are not allowed: opening N with current N+3 executes N→N+1→N+2→N+3 or an explicitly proven equivalent migration registered as such.

# Migration determinism

A migration cannot depend on:

- current UI theme/locale/workspace;
- system clock for semantic values;
- network;
- current display profile;
- random IDs without deterministic migration mapping/recording;
- nondeterministic collection iteration.

If new IDs must be synthesized, derive/map them deterministically from old stable identity + migration namespace where practical, or persist the generated mapping within the single migration result before any retry.

# Save transaction

Explicit Save is a document-session operation against one committed revision.

Canonical sequence:

1. resolve active edit transaction as defined by 09.3;
2. capture immutable committed snapshot + referenced canonical resource snapshot;
3. serialize/encode to a new temp file in the destination filesystem when possible;
4. finalize ZIP central directory;
5. reopen/validate minimum package structure and critical hashes/schema;
6. flush file contents (`fsync`/platform equivalent where practical);
7. atomically replace destination using `aubrieta_platform` semantics;
8. best-effort flush containing directory on platforms where it materially improves durability;
9. only then mark the saved revision as clean.

If any step fails, the old explicit-save file remains authoritative and the document remains dirty. Temp artifacts are cleaned or safely scavenged later.

# Save conflict/external modification

Before replacing an existing file, compare the known file identity/fingerprint/mtime-size token captured at open/last save with current destination state. External modification yields a structured conflict flow (`Save Copy`, `Replace`, `Reload/Compare` where supported); Aubrieta does not silently overwrite a changed external file.

# Save As

`Save As` writes a new native package and only changes the session's canonical file binding after successful validated commit. Failure leaves the original binding and dirty state intact.

# Autosave/recovery architecture

Autosave is not repeated full `Save` disguised under the hood. Maintain a recovery area outside the user's canonical file using document/session ID. Recommended structure:

```
recovery/<document-or-session-id>/
├── recovery-manifest.json
├── checkpoints/
│   └── <revision>.json-or-package
├── blobs/
│   └── content-addressed changed resources/tiles
└── journal/
    └── optional bounded committed-delta records
```

V1 may begin with periodic compact checkpoints + content-addressed changed blobs before optimizing to a finer journal. The recovery format is implementation-private but versioned/tested.

# Autosave snapshot rule

Autosave captures only a **committed** document revision. It never commits an active gesture/IME/brush preview for the user. It can snapshot immutable state concurrently after revision capture.

Autosave frequency combines time + meaningful dirty revision rather than writing when nothing changed. Heavy resource encoding is throttled through JobSystem and must not harm brush/input latency.

# Recovery identity

Recovery metadata records:

- document/session ID;
- original file path token if any;
- explicit-save revision/fingerprint known at checkpoint;
- recovered committed revision;
- checkpoint time for UI information;
- writer/build/recovery schema version;
- resource/blob inventory.

Recovery UI compares identity/revisions, not timestamp alone.

# Recovery open

Opening a recovery candidate produces a normal in-memory document marked **unsaved/recovered**. It never overwrites the original file automatically. Explicit Save/Save As is required to bind durable output.

# Recovery retention/scavenging

Recovery storage has retention/size policy, but never deletes the only newer recoverable state merely because a timer fired while the document still appears crashed/unresolved. Successful explicit save + clean shutdown can mark older checkpoints collectible. Scavenging is bounded, observable and tested after forced process termination.

# Corruption/salvage tiers

Loader distinguishes:

1. container unreadable;
2. manifest invalid;
3. core JSON invalid;
4. individual canonical resource corrupt/missing;
5. optional extension corrupt;
6. derived preview/cache corrupt.

Derived corruption is discarded. Optional extension corruption can isolate that namespace with warning. Individual resource corruption may open placeholders while preserving unaffected document structure. Core-structure corruption enters repair/salvage analysis and never mutates original source.

# Repair output

A repair attempt always writes a **new** `.aubrieta` file/copy and includes a machine-readable repair report: skipped/recovered resources, dropped corrupt optional payloads, unresolved references and validation state. No hidden in-place repair.

# Interchange projection freshness

`interchange/document.pdf` and Surface SVGs record the canonical document revision/generator version they represent. Reader/UI can detect stale/missing compatibility projections. A failed projection refresh does not invalidate a successful canonical Save unless user explicitly configured `requireFreshInterchange=true`; otherwise Save succeeds with warning and stale projection omitted/marked stale.

# Native-format preflight

Before Save, validate core structural invariants, required resources and serialization constraints. Full expensive export-like preflight is not mandatory for native save, but the writer must refuse canonical corruption/non-serializable state rather than writing a package it knows cannot be reopened.

# Security budgets

Before decompression/allocation enforce package-level ceilings for:

- entry count;
- individual compressed/uncompressed size;
- aggregate declared uncompressed size;
- compression ratio;
- JSON depth/token/string lengths;
- raster chunk decoded size;
- ICC/font/image payload limits;
- extension payload limits.

Limits are configurable/testable product constants and use checked arithmetic. Declared lengths are never trusted without verifying decode output.

# Third-party conformance suite

Publish fixtures for:

- minimal valid document;
- each core object/resource type;
- 8/16-bit raster;
- CMYK/ICC/Spot;
- multi-Surface/text flow;
- opaque extension namespace;
- `.aubri` alias;
- missing optional resource diagnostics;
- historical schema migrations.

Reference tooling validates package structure/schema without linking Aubrieta UI/renderer. A third-party reader should be able to inspect document metadata/tree/resources solely from the public spec.

# Additional persistence gauntlets

- process kill at every Save phase, verifying old or new complete file exists, never half-replaced sole file;
- disk full during JSON, raster blob and ZIP finalization;
- permission/read-only destination changes mid-save;
- external modification conflict;
- unknown future optional extension round-trip;
- migration interrupted/retried with identical canonical result;
- corrupt central directory/duplicate entries/path traversal/zip bomb;
- recovery checkpoint while large 16-bit raster changes are occurring;
- successful canonical save with failed optional compatibility projection;
- all historical fixtures opened and resaved by current writer.