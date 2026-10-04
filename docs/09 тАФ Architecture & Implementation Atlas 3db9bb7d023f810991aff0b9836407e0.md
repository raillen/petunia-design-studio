# 09 — Architecture & Implementation Atlas

<aside>
🏛️

**Canonical implementation atlas.** This section expands Aubrieta’s internal architecture to the same no-gap depth required by the Interface Atlas.

</aside>

# Purpose

Every subsystem must document: responsibility, public contracts, owned state, lifecycle, dependency direction, extension points, threading model, persistence, errors/recovery, performance budgets, observability, security boundaries, tests and staged implementation plan.

# Core architectural invariants

- capability-driven modularity;
- dependency inversion at subsystem boundaries;
- canonical document independent of render/UI/export formats;
- actions/commands as authoritative mutation path;
- derived state rebuildable and dependency-tracked;
- resource/token indirection instead of hard-coded UI resources;
- optional capabilities fail closed and degrade predictably;
- every long task is cancelable where consistency permits;
- every persisted schema is versioned and migratable.

# Audit summary

The earlier notebook described intent well but lacked implementation-level contracts in document schema, feature lifecycle, invalidation, persistence/migrations, job scheduling, security, observability, release engineering and extension compatibility. The child pages below close those gaps.

[09.1 — Modularity, Capability Registry & Contribution Architecture](09%201%20%E2%80%94%20Modularity,%20Capability%20Registry%20&%20Contribut%203db9bb7d023f8160a14dea24a4d00544.md)

[09.2 — Canonical Document Model, Object Graph, Resources & IDs](09%202%20%E2%80%94%20Canonical%20Document%20Model,%20Object%20Graph,%20Res%203db9bb7d023f81eea154c0aceec4610b.md)

[09.3 — Actions, Commands, Transactions, Undo/Redo & ChangeSets](09%203%20%E2%80%94%20Actions,%20Commands,%20Transactions,%20Undo%20Redo%20%203db9bb7d023f81dc83e0ef56938b2932.md)

[09.4 — Evaluation Graph, Derived Data, Invalidation & Cache Architecture](09%204%20%E2%80%94%20Evaluation%20Graph,%20Derived%20Data,%20Invalidatio%203db9bb7d023f81a09fe5d8251424e0c7.md)

[09.5 — Vector Geometry Engine: Numeric Policy, Paths, Boolean, Stroke & Snapping Contracts](09%205%20%E2%80%94%20Vector%20Geometry%20Engine%20Numeric%20Policy,%20Path%203db9bb7d023f8151889eddb7e9c44952.md)

[09.6 — Raster Engine: Tiles, Brushes, Masks, Selections & Pixel Storage](09%206%20%E2%80%94%20Raster%20Engine%20Tiles,%20Brushes,%20Masks,%20Select%203db9bb7d023f81dcbd6ecddfbe80a52a.md)

[09.7 — Render Scene, Vello/wgpu Compositor & GPU Resource Lifecycle](09%207%20%E2%80%94%20Render%20Scene,%20Vello%20wgpu%20Compositor%20&%20GPU%20R%203db9bb7d023f81c2bc3ec3d1a5b74cc2.md)

[09.8 — Typography, Text Editing & Layout Engine](09%208%20%E2%80%94%20Typography,%20Text%20Editing%20&%20Layout%20Engine%203db9bb7d023f81c3a4e5ebdeac49ae50.md)

[09.9 — Color Management, ICC, CMYK, Spot & Proofing Architecture](09%209%20%E2%80%94%20Color%20Management,%20ICC,%20CMYK,%20Spot%20&%20Proofin%203db9bb7d023f811d9572d1978cb88e34.md)

[09.10 — Native File Format, Serialization, Versioning, Migrations, Autosave & Recovery](09%2010%20%E2%80%94%20Native%20File%20Format,%20Serialization,%20Version%203db9bb7d023f81b587e8fab1e4d8d339.md)

[09.11 — Import/Export Adapter Contracts, Capability Negotiation & Fidelity](09%2011%20%E2%80%94%20Import%20Export%20Adapter%20Contracts,%20Capabilit%203db9bb7d023f8100a1e8e054604426f9.md)

[09.12 — Resource Packs, Tokens, Themes, Icons, Strings & Configuration Formats](09%2012%20%E2%80%94%20Resource%20Packs,%20Tokens,%20Themes,%20Icons,%20Str%203db9bb7d023f81c5b2a3cbcc67e7c3ee.md)

[09.0 — Documentation Gap Audit & Completeness Matrix](09%200%20%E2%80%94%20Documentation%20Gap%20Audit%20&%20Completeness%20Matr%203db9bb7d023f8110ae91f49bcd016efe.md)

[09.13 — Job System, Concurrency, Cancellation & Priority Scheduling](09%2013%20%E2%80%94%20Job%20System,%20Concurrency,%20Cancellation%20&%20Pr%203db9bb7d023f81ab9dd9d6c95c2f7691.md)

[09.14 — Plugin Host Isolation, WASM Components, Permissions & Versioning](09%2014%20%E2%80%94%20Plugin%20Host%20Isolation,%20WASM%20Components,%20Pe%203db9bb7d023f8160817bd7b0f1061e1e.md)

[09.15 — MCP, Automation, Inspection API & External Control Contracts](09%2015%20%E2%80%94%20MCP,%20Automation,%20Inspection%20API%20&%20External%203db9bb7d023f816fbfaaf12b89582c91.md)

[09.16 — Localization, Locale Formatting, Translation Workflow & Text Resource Architecture](09%2016%20%E2%80%94%20Localization,%20Locale%20Formatting,%20Translati%203db9bb7d023f819ea178cb86bcf6a0d0.md)

[09.17 — Platform Services, Filesystem, Clipboard, Dialogs, Fonts, Pen & OS Integration](09%2017%20%E2%80%94%20Platform%20Services,%20Filesystem,%20Clipboard,%20%203db9bb7d023f814ab768c1aeee5f6f5d.md)

[09.18 — Security Model, Trust Boundaries, Hostile Files & Safe Extension Rules](09%2018%20%E2%80%94%20Security%20Model,%20Trust%20Boundaries,%20Hostile%20%203db9bb7d023f81f696d0cef7cce2a631.md)

[09.19 — Observability, Structured Diagnostics, Logging, Profiling & Crash Bundles](09%2019%20%E2%80%94%20Observability,%20Structured%20Diagnostics,%20Log%203db9bb7d023f8104ab75ec5fb681ef18.md)

[09.20 — Build, Packaging, Release, Updates, Dependency Policy, SBOM & Licensing](09%2020%20%E2%80%94%20Build,%20Packaging,%20Release,%20Updates,%20Depend%203db9bb7d023f81e78381d9774582bcb7.md)

[09.21 — Architecture Governance, ADRs, Compatibility, Implementation Sequence & Definition of Done](09%2021%20%E2%80%94%20Architecture%20Governance,%20ADRs,%20Compatibili%203db9bb7d023f81729bace22cfa3aa2e4.md)

[09.22 — Cargo Workspace, Crate Topology, Dependency Direction & Feature Boundaries](09%2022%20%E2%80%94%20Cargo%20Workspace,%20Crate%20Topology,%20Dependenc%203db9bb7d023f8137949cc85cc5313746.md)

[09.23 — Preferences, Configuration Layers, Workspace State & Persistence Semantics](09%2023%20%E2%80%94%20Preferences,%20Configuration%20Layers,%20Workspa%203db9bb7d023f81a0a63df217a214509c.md)

[09.24 — Application, Document Session, Multi-Window & Shutdown Lifecycle](09%2024%20%E2%80%94%20Application,%20Document%20Session,%20Multi-Windo%203db9bb7d023f8150a1a4dee7fefb025e.md)

[09.25 — Property & Parameter Schema Registry, Generic Inspectors & Bindable Data Contracts](09%2025%20%E2%80%94%20Property%20&%20Parameter%20Schema%20Registry,%20Gene%203db9bb7d023f81918b0cf4b468f588b0.md)

[09.26 — Open ADR Registry, Deferred Decisions & Revisit Triggers](09%2026%20%E2%80%94%20Open%20ADR%20Registry,%20Deferred%20Decisions%20&%20Re%203db9bb7d023f8166a751e3d1a8390b78.md)

[09.27 — UI-Agnostic Core, Ports/Adapters, Presentation Models & Shell Conformance](09%2027%20%E2%80%94%20UI-Agnostic%20Core,%20Ports%20Adapters,%20Presenta%203db9bb7d023f81968082fd246f9ff668.md)

[09.28 — Plugin SDK, Safe Extension Contract, Capability UX & Runtime-Neutral API](09%2028%20%E2%80%94%20Plugin%20SDK,%20Safe%20Extension%20Contract,%20Capab%203db9bb7d023f812b924ffac8a8a3c41d.md)

[09.29 — MCP API Usability, Agent Contracts, Safety, Discovery & Deterministic Automation](09%2029%20%E2%80%94%20MCP%20API%20Usability,%20Agent%20Contracts,%20Safety%203db9bb7d023f8193a933c547bdf2aafe.md)

[09.30 — Plugin Scripting Runtime ADR: Lua vs Python vs JavaScript](09%2030%20%E2%80%94%20Plugin%20Scripting%20Runtime%20ADR%20Lua%20vs%20Python%203db9bb7d023f8129853cc7461081cf62.md)
## Current implementation contract — 2026-10-01

Milestone Required: ADR-002 (`docs/developers/adr/ADR-002-local-path-and-integrity.md`, synchronized pt-BR mirror) governs native schema 2, local path reference sizing, explicit parent input migration, integrity validation, atomic publication, bounded history and package writes. Generated references live in `docs/public/implementation/contracts.json`; execution evidence/status in `docs/developers/implementation-progress.md`. These implemented contracts supersede older coordinate/transaction descriptions on conflict. Runtime is Freya/Skia as pinned in Cargo.lock. Full M0/MVP/V1 and historical GUI authority reconciliation remain open.

Publication integrity includes finite/ranged effect and adjustment chains, unique local entry IDs, checked ID exhaustion, read-only external document access and reversible `SetSurfaceExportEnabled`. Surface placement/layout setters reject invalid inputs before mutation. Raw serde input still requires explicit validation at trust boundaries. The snapshot benchmark accepts explicit workloads and emits structured percentiles; it does not measure full painted frames.

## Modifier-frame contract — schema 3

Milestone Required: ADR-003 (`docs/developers/adr/ADR-003-local-modifier-frames.md`, synchronized pt-BR mirror) supersedes ADR-002 modifier frames/schema version. Persist local reference sizes; migrate schemas 1/2 explicitly; preserve source and parameters on placement edits. Geometry and opacity evaluate in the reference frame. Bake is atomic and preserves world placement/masks; empty crop stays empty. World-space tool conversion and unique guide creation use domain contracts. Shared scene/spatial render, pixel resources and full MVP/V1 remain open.

## MVP render/worker contract continuation — 2026-10-01

Scope: Milestone Required. ADR-004 (`docs/developers/adr/ADR-004-render-snapshots-and-workers.md`, synchronized pt-BR mirror) introduces immutable RenderScene/RenderSurface snapshots, GUI-free antialiased CPU coverage and isolation/mask/effect composition, direct PNG region/DPI output, prepared tonal curves, old/new scene damage and bounded cancellable workers with revision-tagged results. It does not change native schema 3. GUI/glyph/image/tile integration and M0/M2/M3/M4 acceptance remain open. New implementation is UNVALIDATED: the user deferred all tests/gates until every MVP feature has been implemented; historical foundation checks must not qualify the new PR head.


## MVP image continuation — 2026-10-01

Scope: Milestone Required. ADR-005 (`docs/developers/adr/ADR-005-immutable-image-assets.md`, synchronized pt-BR mirror) adds immutable encoded sources/content keys without a schema bump; bounded common codec admission preserving gray RGBA16 and EXIF; shared LRU/pinned-owner accounting and linear-light area pyramids; CPU image composition; atomic placement; a localized import path/error dialog; and bounded Skia uploads with rotation/opacity. ICC/CMYK/HDR and unavailable color/warp capabilities return reasons. Async cold GUI preparation, full scene presentation, binary resources, persistent tiles, glyph caches and M0/M2/M3/M4 acceptance remain open. New implementation is UNVALIDATED; the user's deferred-gate policy still applies.


## MVP text/canvas continuation — 2026-10-01

Scope: Milestone Required. ADR-006 (`docs/developers/adr/ADR-006-shaped-text-and-canvas-preview.md`, synchronized pt-BR mirror) adds advanced uniform-style shaped TTF/CFF outlines, bounded prepared-text caching/nonzero glyph coverage and editable source preservation. Canvas artwork now comes from shared CPU composition in bounded workers, with immutable source identity and complete latest-request publication checks across tabs/cameras/channels; GUI retains one upload and interactive overlays. Indexed job metadata keeps 256 terminal records and canceled queued work releases admission immediately. Flat/default-font artwork painting and per-image uploads were removed. This supersedes prior pending canvas/glyph-preparation descriptions, without completing M1/M2 or changing schema 3. Text-on-path/color/variable adapters, editing/IME/styles/hit-testing, import admission workers, total budgets, persistent bitmap/resources/COW/recovery and Linux/product acceptance remain open; ICC/CMYK/PDF remain V1 Required. All new source is UNVALIDATED under the user's deferred-gate policy; 34 regression cases are prepared, not executed.


## Schema 4 raster and native workflows — implementation pending validation

Scope: Milestone Required. ADR-009 (`docs/developers/adr/ADR-009-persistent-raster-and-native-workflows.md`, synchronized pt-BR mirror) extends ADR-005 through ADR-008: persistent straight RGBA/Gray 8/16-bit sparse COW planes, pressure/selection-aware atomic strokes and cancellable fill; SHA-256 binary PTND resources; exclusive atomic save/export and binary recovery with startup offer; stable-tab I/O workers; whole-subtree native Linux clipboard; persistent uniform text style/fill rules, worker-shaped artistic bounds/font diagnostics; precise scoped SVG import/output; bounded ICC RGB input-to-sRGB derivatives and a transparent CPU export preview. Source preservation and Command publication remain mandatory. This supersedes historical schema-3/no-raster/blanket-ICC-input-rejection pending descriptions. All current changes are UNVALIDATED: the user deferred every gate until all MVP features are implemented. Remaining in-canvas caret/IME, native tablet backend, display-profile configuration, histogram composition, aggregate performance and Linux/product release acceptance remain open. True CMYK/proof/professional PDF stay V1 Required; no MVP completion is claimed.

## 2026-10-03 implementation contract

Current bounded MVP/V1 corrections: [ADR-010](developers/adr/ADR-010-text-histogram-icc-and-pdf.md). Schema 5 carries immutable binary ICC resources; UI drafts and analysis share worker-prepared scenes. PDF/X-4 and native four-plane CMYK remain V1 Required. Release acceptance is not inferred from wiring.


## 2026-10-03 — Native CMYK contract (V1 Required)

ADR-011: `developers/adr/ADR-011-native-cmyk-raster.md`, with pt-BR mirror. Raster 09.6/10.9 uses CMYKA8/16 and validated ICC; persistence 09.13/15.A uses schema 6/index 3. Layer TIFF and regular CMYK ICC PDF preserve samples. Whole-page proof/overprint/DeviceLink/PDF-X remain unavailable. Actual validation is recorded in `public/implementation/native-cmyk-v1.json`.


## ADR-012 — Compact Studio desktop (2026-10-04)

Milestone Required (MVP), shared V1 foundation. `developers/adr/ADR-012-studio-workspace.md` and its pt-BR mirror govern the 08.1/08.2/08.3/08.5/08.6/08.12/08.14/08.17/08.18/08.19/09.16 desktop contracts. `developers/uiux-studio.md` maps the audit; actual final execution is recorded in `public/implementation/uiux-studio.json`. No domain/schema change or complete release acceptance is implied.
