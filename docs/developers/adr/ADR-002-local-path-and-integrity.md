# ADR-002: Local paths and integrity at publication boundaries

- **Status:** Accepted
- **Date:** 2026-10-01
- **Scope:** Milestone Required (MVP)
- **Affects:** document, foundation, application, shell, raster and IO; native schema 2

The modifier-frame decision and schema version are superseded by [ADR-003](/developers/adr/ADR-003-local-modifier-frames); the other contracts remain applicable.

## Context

Schema-1 `Path` points were parent coordinates, while the world resolver rejected them as ambiguous. Moving bounds alone could leave points behind. Independent shape/frame commands could normalize geometry against the wrong frame. Transaction publication could overwrite intervening commands, and failed history replay could discard entries after partly changing the document.

## Decision

1. Persist `LocalPath { path, reference_size }`. Source points are local and immutable during placement edits. Local evaluation scales by current bounds size divided by reference size; world placement applies the object and ancestor matrices. Parametric recipes remain editable.
2. Retain `Path` as an explicitly parent-space compatibility/input descriptor. Mutators and schema-1 loading translate by negative bounds origin and store `LocalPath`. No coordinate heuristic distinguishes old/new points. Unknown schema versions fail. Package/document versions must agree before migration; saves write schema 2.
3. `SetPath` replaces path and frame atomically. Creation commands define bounds before shape. A path-only input without bounds derives a positive frame from its actual geometry; an empty path requires an explicit frame. `to_path` is a compatibility parent-space projection, never the canonical persisted source.
4. Legacy modifier parameters retain their documented parent frame. Local evaluation translates endpoints/quads/crop origin into the local frame; it does not bake the editable chain. A later explicit modifier-frame migration remains required to make all modifier parameters follow placement and scaling.
5. Read/publication validation checks finite positive frames, shape metrics, opacity, strokes, gradient stops, modifier IDs/parameters, surface IDs, global object IDs, reciprocal ownership, same-surface references, acyclic parents, data-source fields and bindings. Graph validation is iterative. Deleting a referenced path detaches text and bindings with reversible changes. Appearance writes validate all current effect/adjustment variants, local entry IDs and documented numeric ranges before touching legacy or stack fields, including disabled entries. Curves permit inverted Y but require increasing X. Mutable document/object/surface handles and schema writes are restricted to the document crate; unused bypass methods were removed. Raw serde decoding remains a low-level untrusted input and still requires validation at persistence/publication boundaries; that validation rejects unmigrated schemas and parent-space paths.
6. Transactions retain their baseline and reject changed live documents. Failed command staging and history replay publish nothing. Undo/redo entries move only after successful replay and validation. Save-point identity follows content history, independently of monotonic render-cache revisions.
7. Default history retains up to 1000 entries and an estimated 512 MiB encoded payload across undo/redo. Counting serialization avoids allocating another payload buffer. Oversized edits fail before publication; retention never silently removes the current edit's undo. This estimate is **not** an RSS limit.
8. Package saves use exclusively created siblings, stream JSON into ZIP, sync the file, atomically replace the destination and sync its directory on Unix. Temporary files are removed by ownership. Reads cap manifest at 64 KiB, document JSON at 256 MiB and archive entry count at 10,000; core entries must be unique. Linked/binary resources, streaming document decoding and recovery remain subsequent work.
9. Raster normalized APIs accept/return straight RGBA; tile storage honors its declared alpha mode. Sixteen-bit channels are little-endian. Skia receives straight upload bytes as Unpremul. Invalid/oversized synchronous brush dabs fail before touching tiles; out-of-range tile addresses cannot wrap.

## Consequences and evidence

Migration, placement/source preservation, atomic replacement/revert, invalid graphs, stale caches, compound-path holes, transaction conflicts, failed undo/redo, save-point branching, budget failure, concurrent saves, package limits, formatter persistence, appearance/layout rejection, export inclusion history and alpha equivalence have dedicated integration tests. Compile-fail doctests prove external mutable object/surface access is unavailable. `cargo xtask migrations` runs the migration/package fixtures. The generated [contract reference](/implementation/contracts.json) lists their source files; the [execution ledger](/developers/implementation-progress) records check results and remaining milestones.

Immutable canonical path buffers are shared by snapshots with Arc, while replacements retain independent source identity. Audited atomic primitives stage in place; compound commands retain a protective copy. The correctness implementation still clones document containers and image resources. That increases mutation cost and memory for large projects. COW resources, operation preconditions/revisions and bounded workers must replace these copies with measured equivalence; this ADR does not claim M0/MVP or V1 completion. Manual Linux/pen/IME/accessibility and print validation remain open.

The audit annex remains immutable baseline evidence. Historical notebooks describing another GUI stack cannot override the current Freya/Skia manifests; the curated architecture now names the implemented runtime. A broader authority-map reconciliation remains an explicit backlog item.
