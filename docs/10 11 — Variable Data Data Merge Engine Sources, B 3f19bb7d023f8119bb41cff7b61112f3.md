# 10.11 — Variable Data / Data Merge Engine: Sources, Bindings, Expressions, Generation & Preflight

# Source interface

IDataSourceProvider -> schema, row count/stream, stable field IDs/names, typed values, refresh fingerprint and diagnostics. Built-ins CSV/JSON/tabular; plugins can register provider.

# Data types

String, number, boolean, date/time, color, image/resource reference, URI/path grant reference and null. Conversion rules explicit; no implicit arbitrary Python objects.

# Binding

BindingId targets ObjectId + PropertyPath and defines SourceExpression + fallback + formatting + null policy. Text can include inline field tokens; images target resource/content properties.

# Expression language

Pure, deterministic, bounded evaluator. Operators, conditional, string/number/date format helpers, field lookup. No filesystem/network/process/Python eval. AST versioned if stored.

# Preview

PreviewContext selects record N and evaluates bindings into **derived preview projection**. Canonical objects are unchanged. Renderer receives preview overrides.

# Validation

Before preview/generate: field existence/type compatibility, expression parse/type, required resource grants. During rows: runtime null/format/path errors associated with record and BindingId.

# Text overflow

Preview/generation checks overset text after bindings and flags record/surface. Policies: error, warn, auto-fit if explicitly configured, allow overflow.

# Image data

Provider returns brokered resource reference. Import/resolve has size/type/security limits. Fit mode per binding.

# Generation modes

1. duplicate Surfaces into current document;
2. create new .PTND document;
3. one .PTND per record;
4. direct export per record/Surface;
5. batch asset generation.

# Job architecture

Generation snapshots template document + source fingerprint + binding set. Iterates rows in bounded batches, can parallelize render/export while maintaining deterministic naming/order. Cancellation stops before future commits/outputs.

# Filename templates

Safe template language; sanitize reserved characters; collision plan shown in preflight. Never allow traversal from data field.

# Preflight

Schema errors, missing fonts/resources, overflow, invalid colors/images, duplicate filenames, export degradations, huge output estimates and unsupported bindings.

# Commands

AddDataSource, RefreshDataSourceMetadata, AddBinding, SetBinding, RemoveBinding. Bulk generated outputs may be separate document/export jobs rather than one enormous undo transaction.

# GUI

Data Merge panel with source/schema/bindings + record navigator. Canvas badges. Generation dialog summarizes outputs and issues.

# Tests

CSV encodings/delimiters, quoted fields, nulls, 100k rows streaming, expression sandbox, filename traversal, image failures, deterministic output order, cancel/restart and source-changed detection.

[10.11.1 — Data Merge Expression Language Grammar, Types & Sandbox](10%2011%201%20%E2%80%94%20Data%20Merge%20Expression%20Language%20Grammar,%20%203f19bb7d023f8179a617e1d681b23dcf.md)

[10.11.2 — Data Source Provider Contract, CSV/JSON Parsing & Refresh Semantics](10%2011%202%20%E2%80%94%20Data%20Source%20Provider%20Contract,%20CSV%20JSON%20%203f19bb7d023f81a9bd78c0e8d8220c5c.md)

[10.11.3 — Binding Targets, Formatting, Null/Fallback & Image/Color Semantics](10%2011%203%20%E2%80%94%20Binding%20Targets,%20Formatting,%20Null%20Fallba%203f19bb7d023f81c1ac71f6b5d750fd56.md)

[10.11.4 — Generation Planner, Naming, Parallelism, Cancellation & Deterministic Outputs](10%2011%204%20%E2%80%94%20Generation%20Planner,%20Naming,%20Parallelism,%203f19bb7d023f81dea40eed10ace1d302.md)