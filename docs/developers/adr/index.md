# Architecture Decision Records

Decisions are first-class documentation. Each ADR records context, decision, consequences and scope status. The full historical registry lives in the canonical notebooks; the records below are the ones every contributor must know first.

| ADR | Title | Status |
| --- | ----- | ------ |
| [ADR-001](/developers/adr/ADR-001-effect-chain) | Non-destructive editing via typed ordered EffectChain | Accepted |
| [ADR-002](/developers/adr/ADR-002-local-path-and-integrity) | Local paths, schema 2 and atomic publication | Accepted · Milestone Required |
| [ADR-003](/developers/adr/ADR-003-local-modifier-frames) | Persistent local modifier frames, schema 3 and appearance-preserving bake | Accepted · Milestone Required |
| [ADR-004](/developers/adr/ADR-004-render-snapshots-and-workers) | Immutable vector render snapshots and bounded workers | Accepted contract · Milestone Required · validation pending |
| [ADR-005](/developers/adr/ADR-005-immutable-image-assets) | Immutable image sources, bounded decoding and shared display pyramids | Accepted contract · Milestone Required · validation pending |
| [ADR-006](/developers/adr/ADR-006-shaped-text-and-canvas-preview) | Shaped text outlines and latest-request canvas presentation | Accepted contract · Milestone Required · validation pending |
| [ADR-007](/developers/adr/ADR-007-desktop-file-workflows) | Desktop file workflow adapter | Accepted contract · Milestone Required · see bounded UI checks |
| [ADR-008](/developers/adr/ADR-008-object-edit-drafts) | Stable object edit drafts | Accepted contract · Milestone Required · bounded UI checks |
| [ADR-009](/developers/adr/ADR-009-persistent-raster-and-native-workflows) | Persistent raster, binary resources and native workflows | Accepted contract · Milestone Required · validation pending |
| [ADR-010](/developers/adr/ADR-010-text-histogram-icc-and-pdf) | Draft text, composition histograms, ICC resources and faithful PDF | Accepted contract · Milestone Required / V1 Required · final evidence separate |

| [ADR-011](/developers/adr/ADR-011-native-cmyk-raster) | Native CMYK raster, ICC resources and ink-preserving interchange | Accepted contract · V1 Required · final evidence separate |

## Writing a new ADR

1. Copy the ADR-001 file header pattern (`Status`, `Date`, `Scope`, `Context`, `Decision`, `Consequences`).
2. Name it `ADR-<NNN>-<kebab-title>.md` (next free number).
3. Add a row to this index **and** its [pt-BR mirror](/pt/developers/adr/) in the same change.
4. Link it from the page whose behavior it governs.

An ADR without a scope status (`V1 Required`, `Milestone Required`, `Post-V1 Candidate`, `Research`, `Open ADR`, `Historical`, `Out of Scope`) is a draft, not a decision.
