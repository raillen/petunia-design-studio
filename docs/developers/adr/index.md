# Architecture Decision Records

Decisions are first-class documentation. Each ADR records context, decision, consequences and scope status. The full historical registry lives in the canonical notebooks; the records below are the ones every contributor must know first.

| ADR | Title | Status |
| --- | ----- | ------ |
| [ADR-001](/developers/adr/ADR-001-effect-chain) | Non-destructive editing via typed ordered EffectChain | Accepted |

## Writing a new ADR

1. Copy the ADR-001 file header pattern (`Status`, `Date`, `Scope`, `Context`, `Decision`, `Consequences`).
2. Name it `ADR-<NNN>-<kebab-title>.md` (next free number).
3. Add a row to this index **and** its [pt-BR mirror](/pt/developers/adr/) in the same change.
4. Link it from the page whose behavior it governs.

An ADR without a scope status (`V1 Required`, `Milestone Required`, `Post-V1 Candidate`, `Research`, `Open ADR`, `Historical`, `Out of Scope`) is a draft, not a decision.
