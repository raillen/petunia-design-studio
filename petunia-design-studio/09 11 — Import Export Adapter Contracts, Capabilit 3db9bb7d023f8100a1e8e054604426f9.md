# 09.11 — Import/Export Adapter Contracts, Capability Negotiation & Fidelity

# Adapter principle

External formats are translators around the canonical document. Importers/exporters advertise explicit capabilities and loss/degradation behavior.

# Import contract

Sniff type independently from extension when safe; enforce resource/decompression limits; parse to intermediate adapter model if useful; normalize/validate; create one atomic import transaction. Partial import only under explicit recovery mode.

# Export contract

Exporter receives immutable document/evaluated snapshot + ExportOptions + cancellation token + diagnostics sink. It never reads UI widgets or viewport screenshots.

# Capability descriptors

Per format: vector paths, text, fonts, images, CMYK, ICC, spot, masks, clipping, blend modes, gradients, effects, multiple surfaces/pages, metadata, external links. Unsupported feature has policy: preserve, approximate, rasterize, expand or reject.

# Preflight/degradation plan

Before export, derive a list of degradations with affected IDs. UI may preview/confirm according to severity; batch export can apply saved policy.

# PDF

Krilla primary high-level writer; pdf-writer escape hatch. Preserve vector/text/color semantics where supported. **PDF/X is explicitly deferred outside V1 native scope**: Aubrieta produces professional color-managed PDF plus preflight, while PDF/X conversion/validation may be external until the ordinary PDF/CMYK/ICC pipeline is production-stable.

# SVG

usvg is normalization/import prior art; Aubrieta exporter maps its own semantics. SVG unsupported effects get explicit rasterize/expand policy. SVG is the preferred cross-editor **editable vector projection** and one SVG may be generated per Surface inside the native compatibility bundle.

# Raster export

Region/surface selection, dimensions/resampling, bit depth, alpha, color profile/embed policy and metadata stripping/preservation.

# Security

Importers are hostile-input boundaries with fuzzing, file-size/decompression budgets and no network resolution by default.

# Open native-format interoperability

The `.aubrieta` package carries derived PDF/SVG compatibility representations while preserving the richer canonical Aubrieta model separately. Direct `Save/Export as SVG` and `Save/Export as PDF` remain first-class collaboration paths. External-format roundtrip never claims native fidelity when unsupported Aubrieta semantics were expanded/rasterized.

# Tests

Roundtrip where meaningful, external fixture suites, malformed files, deterministic export, degradation diagnostics, cancellation, huge resource limits, and opening generated SVG/PDF fixtures in representative external editors.

# Adapter lifecycle

Import/export adapters are capability providers registered through 09.1. Every adapter declares stable `ImporterId`/`ExporterId`, supported media types/extensions, sniffing confidence rules, schema/capability descriptor, security limits, cancellation support, fidelity/degradation matrix and version.

Built-in and plugin adapters use the same discovery surface where practical. UI must not special-case a format by concrete crate type.

# File-type detection

Detection order:

1. explicit user-selected format when Save/Export determines it;
2. strong magic/media/package signature;
3. validated structural sniffing;
4. extension as hint/fallback.

A mismatch between extension and strong content signature is surfaced; loader does not blindly dispatch by suffix.

Sniffing reads a bounded prefix/range and cannot decompress/unpack arbitrary amounts before security budgets are established.

# Import phases

Canonical import pipeline:

```
Acquire bounded input
 → Detect format
 → Parse to adapter-owned intermediate model
 → Validate/sanitize hostile structures
 → Analyze capabilities/degradations/ambiguities
 → Resolve ImportOptions / Interpretation Plan
 → Convert to Aubrieta semantic staging model
 → Validate staging model
 → Commit one canonical document transaction
 → Emit import report
```

Parsing/conversion never mutates `DocumentStore` incrementally. A failed import leaves destination document unchanged unless user explicitly invoked a documented salvage/partial-import mode.

# Import staging model

Adapters may define private parser ASTs, but conversion should target Aubrieta-owned semantic staging DTOs rather than directly constructing private storage internals. Staging preserves source identifiers/provenance sufficient for diagnostics and link/resource resolution.

Import resource deduplication uses content fingerprints + semantic metadata when safe; it never merges two distinct editable resources solely because filenames match.

# Import interpretation

Ambiguities that materially affect fidelity/editability become typed `ImportDecision` entries. Examples:

- text preserve vs outline;
- unsupported effect rasterization;
- page/Surface subset;
- linked vs embedded resource;
- untagged/missing color-profile policy;
- source DPI for unavoidable rasterization.

Each decision has a safe recommended default, explanation `TextId`, alternatives and whether it can be applied globally/per item. Headless/MCP import can supply a policy preset; unresolved required decisions return `NeedsInput` rather than guessing.

# Import report

A successful import returns structured report with:

- source format/version detected;
- objects/resources created;
- warnings/degradations;
- substitutions (fonts/profiles);
- unsupported/skipped source features;
- linked-resource results;
- source→Aubrieta ID mapping where useful for automation/diagnostics.

This report can drive UI summary and tests without parsing log strings.

# Export snapshot

Exporter receives an immutable snapshot bound to one committed document revision, explicit target set and evaluated-output quality context. Long export continues from that snapshot even if the user keeps editing; UI identifies revision/dirty-state if relevant. Export never locks the entire document for its full duration merely to keep state stable.

# Export phases

```
Resolve target/format/options
 → Build capability/fidelity analysis
 → Produce DegradationPlan + PreflightReport
 → Resolve required policy decisions
 → Capture/finalize immutable evaluated snapshot
 → Encode to temp output(s)
 → Validate writer result where practical
 → Atomic destination commit / conflict policy
 → ExportReport
```

For batch export, each target output has independent result while the batch owns shared cancellation/progress policy.

# Capability descriptor model

Capabilities must be granular and machine-readable, not a single `supports_text=true`. Example domains:

- geometry: cubic paths, compound paths, boolean-live vs baked, strokes/variable strokes;
- text: editable text, text frames, text-on-path, variable fonts, OpenType features;
- raster: bit depths, alpha, CMYK, profiles;
- appearance: gradients, patterns, blend modes, opacity, masks/clips, effects;
- color: RGB/CMYK/Lab/Gray/Spot/Registration, ICC embedding;
- document: multiple Surfaces/pages, bleed, metadata, links;
- resources: embedded images/fonts/profiles;
- automation/data bindings/extension data.

Capability values may be `Native`, `RepresentableWithConstraints`, `RequiresExpansion`, `RequiresRasterization`, `Unsupported`, plus typed constraints.

# Fidelity grades

Use explicit severity/grade in preflight:

- `Exact` — target can preserve semantic/editable meaning expected by contract;
- `EquivalentAppearance` — structure changes but intended appearance should remain;
- `Approximate` — visible/semantic differences possible within documented bounds;
- `DestructiveDegradation` — rasterize/expand/flatten or lose editability;
- `Unsupported` — cannot produce acceptable target without rejecting/omitting.

A format/export preset maps grades to policy (`allow`, `warn`, `require confirmation`, `reject`). Batch/headless workflows must set this policy explicitly.

# Degradation item

Each item identifies:

- stable diagnostic/degradation code;
- affected object/resource IDs;
- source feature;
- target capability constraint;
- proposed strategy;
- fidelity grade/severity;
- previewability;
- alternative strategies if available.

No exporter hides degradation only in a console log.

# Destructive conversion boundary

Export-only expansion/rasterization happens in the immutable export evaluation graph/staging scene and **does not mutate the source document**. The source changes only if the user invokes an explicit document command like `Expand`, `Rasterize` or `Convert to Curves`.

# Deterministic export

Given the same committed snapshot, export options, adapter/evaluator versions and referenced resource bytes, logical output should be deterministic modulo explicitly documented container metadata (timestamps/IDs added by target format). Avoid hash-map iteration order/randomness in object/page/resource ordering.

Tests may canonicalize target files before byte comparison where target format permits irrelevant nondeterminism.

# Output transaction

Writers create temp files/directories through platform service and only move to final destination after successful encode/validation. Multi-file outputs (SVG + assets, batch variants) use a staging directory/manifest so failures do not leave a misleading partially complete set unless policy explicitly allows partial batch completion.

Overwrite policy is resolved before final commit and external destination changes are checked where feasible.

# Cancellation

Parser/evaluator/encoder loops poll cancellation at bounded intervals. Cancellation before destination commit removes staging artifacts and returns `Cancelled`; it does not produce a file that looks complete. If cancellation arrives during an OS-level atomic finalization that cannot be interrupted safely, complete that tiny critical section then report final state accurately.

# Network/external resource policy

Imports do not fetch remote URLs by default. External links are preserved/represented as links or marked unresolved according to format. Network access requires explicit user/capability policy through brokered platform/network service.

SVG/XML/PDF/resource parsers must disable active scripting/external entity/unsafe URI behavior unless a narrowly reviewed importer requirement explicitly opts in through safe broker.

# Font handling

Import preserves requested font identity and reports substitution. Export embeds/subsets/outlines only according to target/user policy and font embedding permissions. Missing fonts are preflight facts, never silently converted to curves without policy.

# Color handling

Importer/exporter uses `aubrieta_color`; adapters do not implement private profile conversion semantics. Untagged input follows typed ImportColorPolicy. Output conversion/preserve-numbers/spot policies are part of ExportOptions/Preflight.

# Rasterization context

When degradation requires rasterization, adapter requests a final-quality renderer snapshot with explicit:

- pixel bounds;
- resolution/DPI/scale;
- bit depth;
- background/alpha policy;
- working/output profile;
- effect padding;
- antialias/quality mode.

No export adapter samples the interactive viewport or inherits current zoom accidentally.

# PDF V1 contract

V1 target is high-quality professional PDF, not PDF/X certification. Required acceptance is defined feature-by-feature in Functional Atlas/preflight. If a requested print workflow requires PDF/X, Aubrieta communicates that external conversion/validation is required rather than labeling ordinary PDF as compliant.

# SVG V1 contract

SVG exporter prioritizes editable standard constructs. Aubrieta namespaced metadata may aid reimport but cannot be required for ordinary standards-compliant display. External references/scripts are not emitted unless an explicit safe export option exists; default is self-contained resources where practical.

# Raster V1 contract

PNG/JPEG/TIFF output must make depth/profile/alpha capabilities explicit. 16-bit capable source exported to an 8-bit-only target triggers an explicit conversion/degradation entry rather than silent truncation.

# Plugin adapter safety

Third-party importer/exporter is invoked through Plugin SDK capability broker with bounded input/output handles and declared permissions. A plugin cannot obtain arbitrary filesystem/network access simply because it implements an exporter. Host performs final destination commit where feasible.

# Progress model

Adapters report structured hierarchical progress phases (`parse`, `convert`, `evaluate`, `encode`, `validate`, `commit`) with optional item counts/bytes. UI never fabricates percentage when adapter only knows phase/indeterminate progress.

# Export/Import reports

Result objects contain status, source/target, revision, adapter version, outputs, diagnostics, degradation summary, timing and deterministic correlation ID. Reports are machine-readable for MCP/code-agent gauntlets.

# Additional gauntlets

- mislabeled extension with valid/invalid magic;
- parser failure after large partial parse leaving document unchanged;
- required ImportDecision in headless mode returning `NeedsInput`;
- export while source document continues editing, proving snapshot isolation;
- cancellation at every phase and no false-complete output;
- destructive export degradation leaving source untouched;
- 16-bit/CMYK/Spot fidelity policies;
- malformed external references/entities/scripts blocked;
- plugin adapter denied filesystem/network capability;
- multi-file output failure during finalization;
- deterministic output ordering across repeated runs.