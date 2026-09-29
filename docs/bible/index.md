# Constitution (SSOT)

The non-negotiable core of Petunia Design Studio. Everything else — tools, panels, shells, docs — is derived from this page. On conflict, this page wins; the conflict is recorded, not silently resolved.

## I. Identity

- **I-1.** The product is **Petunia Design Studio**. Legacy names (Aubrieta) are read-only history, never emitted.
- **I-2.** The native format is **`.ptnd`** (open ZIP package). Legacy `.aubrieta`/`.aubri` open; new saves emit `.ptnd`.
- **I-3.** English is the canonical documentation language; [pt-BR](/pt/bible/) mirrors every page 1:1.

## II. Mutation pipeline (inviolable)

- **II-1.** Every mutation flows **UI / Shortcut / Plugin / MCP → Action → Command → DocumentMutator → ChangeSet.**
- **II-2.** Nothing touches document storage directly. `DocumentMutator` is the sole writer.
- **II-3.** One gesture = one undo. Preview writes no history; `Up` commits once.

## III. Non-destruction

- **III-1.** Editing is non-destructive by default via the typed ordered **EffectChain** ([ADR-001](/developers/adr/ADR-001-effect-chain)).
- **III-2.** Bake, Expand, Rasterize and Convert-to-Curves are **explicit user operations only**. No silent bake, no silent deletion.
- **III-3.** Base vs. evaluated: tools edit base geometry; render/hit/selection/export read the evaluated result.

## IV. Boundaries

- **IV-1.** Domain crates never import GUI toolkit types. UI receives DTOs/view-models; it sends `ActionRequest`/`CommandRequest` across the bridge.
- **IV-2.** Features compose through capability registries. A missing capability is a normal disabled state **with a reason** — never a panic, never a dead button.
- **IV-3.** Identity is typed and stable (`ObjectId`, `SurfaceId`, `ResourceId`, …). A Vec index, pointer or handle is never identity.

## V. Honesty

- **V-1.** No fake UI: unimplemented behavior is implemented, disabled-with-reason, hidden, or marked experimental.
- **V-2.** Compile ≠ implemented. Skipped test ≠ pass. Degraded export (e.g. sampled opacity) is logged, never silent.
- **V-3.** Every feature carries explicit scope status. A bare future/later/planned authorizes nothing.

## VI. Verification

- **VI-1.** Done = focused tests + applicable gauntlets + recorded evidence (headless-first, detach proof, token/a11y where UI).
- **VI-2.** Contracts change with their docs: Atlas, ADR registry, references **and EN+pt-BR pages** update in the same change.

First formalization of section VI as build-enforced spec: [SPEC-001](/bible/SPEC-001).
