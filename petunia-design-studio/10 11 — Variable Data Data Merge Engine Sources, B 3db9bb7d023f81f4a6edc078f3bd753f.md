# 10.11 — Variable Data / Data Merge Engine: Sources, Bindings, Expressions, Generation & Preflight

# Data source abstraction

`DataSourceAdapter` provides schema + records/pages/stream + stable field IDs/types + refresh capability. **V1_REQUIRED built-ins:** CSV, TSV and JSON. SQLite, REST/API and Google Sheets adapters are **POST_V1_CANDIDATE**; plugin-provided adapters use the versioned DataSource capability when explicitly installed.

# Field types

**V1_REQUIRED field types:** Text, Number, Date/DateTime, Boolean, Image/File reference and URL. Additional semantic field types require a versioned schema/capability extension and an explicit scope status. Parsing errors retain source row/field location.

# Binding

`DataBinding` links DataSourceId/FieldId (or expression) to target ObjectId + property path + formatter/policy. Bindings survive object reorder and UI changes.

# Supported targets

Text content, image source, selected numeric/color/style properties when explicitly declared bindable. Property schemas advertise bindability; no arbitrary reflection into private structs.

# Formatting

Locale-aware number/date/currency/string formatters with explicit locale/default fallback. Formatting does not mutate source data.

# Conditions

Visibility condition/expression is a constrained validated expression language or structured rule tree. No arbitrary script execution in baseline Data Merge.

# Preview

Preview selects record and evaluates a derived document view without committing generated values into source template. Editing template stays possible while previewing.

# Generation

Modes: materialize Surfaces/objects, export one file per record, multipage output where adapter supports, or batch asset outputs. Generation uses immutable template snapshot + record + cancellation and deterministic filename template.

# Overflow/preflight

Detect text overflow, missing images, invalid values, output filename collisions and failed color/format conversion per record. UI can skip/fail/collect according to policy.

# Refresh

Source refresh diffs schema/records. Removed/renamed fields mark bindings unresolved but preserved for relink.

# Tests

Quoted CSV/newlines/encoding, large record counts streaming, locale formatting, missing image, field schema change, preview isolation, filename collision, cancellation and deterministic output.

# Implementation contract — V1

## Scope status

**V1_REQUIRED data sources:** CSV, TSV and JSON from local/brokered files; schema inference/override; stable field IDs; preview; text/image/bindable-property bindings; locale-aware formatters; record navigation; batch generation/export; preflight; streaming/cancellation.

**POST_V1_CANDIDATE:** SQLite, REST/API, Google Sheets and other network-backed sources unless supplied later through an accepted plugin capability; advanced arbitrary expression language; complex conditional visibility/layout rules. V1 must not ship disabled or placeholder controls for these deferred features.

## DataSource identity

A source has stable `DataSourceId`, adapter/type ID, configuration, content fingerprint/revision, schema version and field descriptors. Field identity is not the display column title alone. For tabular sources, preserve a stable internal FieldId mapping derived from source schema/import configuration so renaming a display label can be relinked deliberately instead of silently changing bindings.

## Record identity

Records expose stable source-relative `RecordKey` when the source provides one; otherwise Aubrieta may derive a deterministic import-session key from row position/fingerprint with clear limitations. Generation filenames/history/logs should refer to RecordKey, not only UI row number.

## Parsing and schema

Parsing is staged before it affects document bindings. Import settings include encoding, delimiter/quote rules where relevant, header policy, locale parsing policy and optional explicit type overrides. A value that fails typed parsing remains a structured field error with row/field/source location; it is not silently coerced to empty text.

Schema inference samples under bounded rules and produces a preview; user confirmation/override is persisted with DataSource configuration. Refresh uses the persisted parsing contract rather than rerunning uncontrolled inference each time.

## Security and file/image references

Source files and record-linked images/files are untrusted input. Resolve relative paths only through the source's granted directory/root policy; prevent `..`/absolute escape unless separately granted. Network URLs are data values only in V1 unless an adapter with explicit network permission supports fetching them. Data Merge cannot become an implicit arbitrary downloader.

## Binding contract

A `DataBinding` contains stable BindingId, DataSourceId, FieldId or approved formatter pipeline, target ObjectId, PropertyId/subpath, expected target type, coercion policy, missing-value policy and optional formatting metadata. Binding validation happens when created and again during preview/generation.

Bindings never store translated property labels or UI panel paths as identity.

## V1 formatting

Use safe, declarative formatter descriptors for text, number, date/time and currency/localized display where types permit. Formatting is pure: `source value + formatter + locale/context → target value/text`. The formatter cannot mutate documents, access files/network or execute script.

## Conditions/expressions status

The earlier “condition/expression” concept is **POST_V1_CANDIDATE** beyond simple formatter/default-value rules. Do not introduce an ad hoc expression language during V1. If basic visibility-by-empty/nonempty is later promoted, define it first as a constrained structured rule schema with explicit operators and no script execution. Lua plugins may perform automation externally but are not hidden expression evaluators inside the baseline Data Merge document model.

## Preview isolation

Record preview creates a derived evaluation overlay/view over an immutable template revision. It does not write bound values into canonical template objects, create undo history or mark the document dirty merely by changing the preview record.

While preview is active, normal template edits remain Commands against the template. Preview re-evaluates against the new revision; stale record evaluation is discarded.

## Generation snapshot

Starting Generate captures:

- template/document revision or immutable snapshot;
- source revision/fingerprint and schema;
- selected record set/order;
- active bindings/formatters;
- output preset/path grant;
- naming policy and collision behavior.

Generation is a Job. Later template/source changes do not silently alter an already-running batch.

## Output modes

V1 supports deterministic variants of:

1. materialize generated Surfaces/objects into the current/new document;
2. export one output per record;
3. multipage/multi-Surface export when the chosen exporter supports it.

Each mode declares whether a failed record aborts all output, skips that record or collects failures. Default interactive policy should preflight first and avoid silent partial success.

## Filename templates

Filename formatting uses a safe structured template with field substitutions and sanitization. Reject/escape path separators and platform-illegal names. Detect collisions before write when feasible and apply explicit `fail`, `number`, `overwrite-with-confirmation` or preset policy. No source value can escape the granted output directory.

## Preflight

Per-record findings include unresolved bindings, parse/coercion errors, overset text, missing/unreadable image, unsupported property value, export degradation, output collision and resource limit. Aggregate summary must let users navigate back to affected record/ObjectId/PropertyId.

## Large datasets

Adapters expose pagination/streaming rather than requiring all records in RAM. Preview fetches bounded windows. Generation applies backpressure between source decoding, document evaluation and exporter jobs. Cancellation stops new records and safely completes/abandons current staged output according to adapter contract.

## Refresh/relink

Refresh compares source fingerprint/schema and returns a structured diff: added/removed/renamed/changed-type fields. Bindings to missing fields enter `Unresolved` state but preserve original IDs/names/config for manual relink. Automatic relink is allowed only on high-confidence stable identity, never approximate name guessing without user confirmation.

## Plugin/MCP API

Plugin `DataSourceAdapter` uses the same schema/record contract and explicit permissions. MCP can list sources, schemas, bindings, preview record/preflight and start/cancel generation Jobs without accessing arbitrary host paths. All methods are documented in beginner-oriented cookbook form.

## Required tests

Add encoding/delimiter configs, deterministic FieldId mapping, type-override persistence, malicious relative paths, preview-not-dirty, preview under template edit, immutable generation snapshot, per-record failure policy, output path escape/collision, million-row streaming/backpressure, refresh schema diff/relink, cancel mid-batch and UI↔MCP/plugin adapter parity.