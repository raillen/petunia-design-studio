# 10.11.2 — Data Source Provider Contract, CSV/JSON Parsing & Refresh Semantics

# Provider

[IDataSourceProvider.open](http://IDataSourceProvider.open)(grant/resource, options) returns schema, fingerprint, row access/iterator and diagnostics.

# Fields

Stable FieldId/name, inferred/declared type, nullable and metadata. Duplicate column names disambiguated.

# CSV

Encoding/options, delimiter, quote/escape, header, locale number/date policy; streaming bounded parser.

# JSON

Array-of-objects baseline; nested flattening/root selection only through bounded declarative rule.

# Refresh

Fingerprint change reparses schema; bindings remap by stable field identity/name policy and broken bindings are reported.

# Large

100k+ rows stream/page; preview loads only needed records.

# Tests

Quoted CSV/newlines/encodings, schema change, duplicate names, huge rows, malformed JSON and permission loss.