# 12.5 — Living Documentation, Schemas, API Reference, Examples & Bilingual Publishing

# Documentation classes

Canonical architecture, functional contracts, API/schema reference, user guide, contributor guide, ADRs, evidence reports and historical migration notes are distinct.

# Canonical vs generated

Notion can be planning/authority notebook; repository contains versioned implementation-facing docs/schemas. Stable claims must have a reproducible path between authorities. Generated API docs originate headers/schema metadata, not hand-copied signatures.

# API reference

Document native Python facade, plugin SDK, MCP methods, Action/Property schemas and PTND format. Every method/action includes intent, inputs, outputs, errors, permissions, side effects, undo/revision and examples.

# Schema source

JSON Schema/IDL/typed metadata lives in repository and is versioned. Docs embed generated tables/examples. CI detects stale generated output.

# Examples

Maintain minimal “hello” plus realistic end-to-end examples: create vector artwork, edit photo nondestructively, export/preflight, plugin panel/tool, MCP workflow, PTND inspection.

# Languages

Canonical developer docs may be en-US with mandatory pt-BR release projection, matching prior Petunia policy. UI strings remain localized resource system independent from docs.

# Drift

A code change touching schema/API/tool/action must identify documentation impact. CI/change template can fail if required doc/evidence field absent.

# Historical docs

Rust/Slint pages remain linked as historical source only, visibly superseded. No agent should route to them as current implementation contract.