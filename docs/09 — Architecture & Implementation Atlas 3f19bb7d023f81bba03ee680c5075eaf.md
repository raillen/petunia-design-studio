# 09 — Architecture & Implementation Atlas

# Purpose

Especificar internals de implementação no mesmo nível de rigor do Interface Atlas.

# Invariantes

- core C++ UI-agnostic;
- Python application layer estritamente tipada;
- bindings grossos, versionados e testáveis;
- Action/Command é única via de mutação;
- IDs estáveis;
- derived data descartável;
- jobs canceláveis;
- nenhum hot loop obrigatório em Python;
- plugins/MCP passam pelo mesmo semantic core;
- formato .PTND não depende de Qt/Python;
- dependency direction validada em CI.

# Required architecture evidence

Cada subsystem deve documentar:

- ownership;
- inputs/outputs;
- error model;
- memory model;
- threading;
- persistence;
- invalidation;
- extension points;
- performance budgets;
- security boundary;
- tests;
- implementation sequence.

[09.1 — Modularity, Capability Registry & Contribution Architecture](09%201%20%E2%80%94%20Modularity,%20Capability%20Registry%20&%20Contribut%203f19bb7d023f81658f17c1551ebbf7ab.md)

[09.2 — Canonical Document Model, Object Graph, Resources & Stable IDs](09%202%20%E2%80%94%20Canonical%20Document%20Model,%20Object%20Graph,%20Res%203f19bb7d023f812e8136d091b1fe2eda.md)

[09.3 — Actions, Commands, Transactions, Undo/Redo & ChangeSets](09%203%20%E2%80%94%20Actions,%20Commands,%20Transactions,%20Undo%20Redo%20%203f19bb7d023f81a29701d62aaef8a0bd.md)

[09.4 — Python/C++ Boundary, nanobind, Ownership, Lifetime, GIL & ABI](09%204%20%E2%80%94%20Python%20C++%20Boundary,%20nanobind,%20Ownership,%20L%203f19bb7d023f81c89b75ed0dff5735ba.md)

[09.5 — Evaluation Graph, Derived Data, Invalidation & Cache Architecture](09%205%20%E2%80%94%20Evaluation%20Graph,%20Derived%20Data,%20Invalidatio%203f19bb7d023f81f191bee77c7379c6ac.md)

[09.6 — Vector Geometry Engine: Paths, Booleans, Strokes, Hit Testing & Snapping](09%206%20%E2%80%94%20Vector%20Geometry%20Engine%20Paths,%20Booleans,%20Str%203f19bb7d023f81b8a4f6cac181c4c92d.md)

[09.7 — Raster Engine: Tiles, Brushes, Masks, Selections & Pixel Storage](09%207%20%E2%80%94%20Raster%20Engine%20Tiles,%20Brushes,%20Masks,%20Select%203f19bb7d023f81dfa8accfaf69bbc4fb.md)

[09.8 — Render Scene, GPU Backend, Compositor, Shaders & Device Loss](09%208%20%E2%80%94%20Render%20Scene,%20GPU%20Backend,%20Compositor,%20Shad%203f19bb7d023f813abcf9f4093afd3ea6.md)

[09.9 — Typography, Text Editing, Shaping & Layout Engine](09%209%20%E2%80%94%20Typography,%20Text%20Editing,%20Shaping%20&%20Layout%20%203f19bb7d023f81faa35dfcaf9f80023c.md)

[09.10 — Color Management, ICC, CMYK, Spot, Proofing & Display Pipeline](09%2010%20%E2%80%94%20Color%20Management,%20ICC,%20CMYK,%20Spot,%20Proofin%203f19bb7d023f81f29787fbd5f10d4ed2.md)

[09.11 — .PTND Native File Format, Serialization, Migrations, Autosave & Recovery](09%2011%20%E2%80%94%20PTND%20Native%20File%20Format,%20Serialization,%20Mi%203f19bb7d023f810ab9dad5fb9ac75059.md)

[09.12 — Import/Export Adapters, Capability Negotiation, Fidelity & Preflight](09%2012%20%E2%80%94%20Import%20Export%20Adapters,%20Capability%20Negotia%203f19bb7d023f818989ebfdb07406e8d8.md)

[09.13 — Job System, Concurrency, Cancellation, Priority & Backpressure](09%2013%20%E2%80%94%20Job%20System,%20Concurrency,%20Cancellation,%20Pri%203f19bb7d023f8145ad8adf12de4e0000.md)

[09.14 — Plugin Architecture: Python Host, Sandboxing, Permissions, SDK & Packaging](09%2014%20%E2%80%94%20Plugin%20Architecture%20Python%20Host,%20Sandboxin%203f19bb7d023f819b9a09f3a6a3e40281.md)

[09.15 — MCP API: Discovery, Semantic Automation, Inspection, Jobs & Safety](09%2015%20%E2%80%94%20MCP%20API%20Discovery,%20Semantic%20Automation,%20In%203f19bb7d023f814fb512e3688ecf24d3.md)

[09.16 — Platform Services, Filesystem, Security, Diagnostics & Crash Handling](09%2016%20%E2%80%94%20Platform%20Services,%20Filesystem,%20Security,%20D%203f19bb7d023f81ee98d0d676f108ad12.md)

[09.17 — Build, Packaging, CI/CD, Release, Updates, SBOM & Licensing](09%2017%20%E2%80%94%20Build,%20Packaging,%20CI%20CD,%20Release,%20Updates,%203f19bb7d023f810d857fc0dabe6978ce.md)

[09.18 — Repository Topology, Clean Architecture, Python Typing, C++ Safety & ADR Governance](09%2018%20%E2%80%94%20Repository%20Topology,%20Clean%20Architecture,%20P%203f19bb7d023f81b29906e789f98b907e.md)

[09.19 — Observability, Structured Diagnostics, Logging, Profiling & Crash Bundles](09%2019%20%E2%80%94%20Observability,%20Structured%20Diagnostics,%20Log%203f19bb7d023f8132937ad5767351f8dc.md)

[09.20 — Build, Packaging, Release, Updates, Dependency Policy, SBOM & Licensing](09%2020%20%E2%80%94%20Build,%20Packaging,%20Release,%20Updates,%20Depend%203f19bb7d023f81cda299d93d3d4890a0.md)

[09.21 — Architecture Governance, ADRs, Compatibility, Implementation Sequence & Definition of Done](09%2021%20%E2%80%94%20Architecture%20Governance,%20ADRs,%20Compatibili%203f19bb7d023f811c9523cf37d1c9c094.md)

[09.22 — CMake Targets, Python Package Topology, Dependency Direction & Feature Boundaries](09%2022%20%E2%80%94%20CMake%20Targets,%20Python%20Package%20Topology,%20De%203f19bb7d023f811f95c8d367cb2dc3b8.md)

[09.23 — Preferences, Configuration Layers, Workspace State & Persistence Semantics](09%2023%20%E2%80%94%20Preferences,%20Configuration%20Layers,%20Workspa%203f19bb7d023f811db53ece7cfd9e6812.md)

[09.24 — Application, Document Session, Multi-Window & Shutdown Lifecycle](09%2024%20%E2%80%94%20Application,%20Document%20Session,%20Multi-Windo%203f19bb7d023f811488d0d049644d2af1.md)

[09.25 — Property & Parameter Schema Registry, Generic Inspectors & Bindable Data Contracts](09%2025%20%E2%80%94%20Property%20&%20Parameter%20Schema%20Registry,%20Gene%203f19bb7d023f81838321fbc0d0e2fbff.md)

[09.26 — Open ADR Registry, Deferred Decisions & Revisit Triggers](09%2026%20%E2%80%94%20Open%20ADR%20Registry,%20Deferred%20Decisions%20&%20Re%203f19bb7d023f816b908ffe078e6419f2.md)

[09.27 — UI-Agnostic Core, Ports/Adapters, Presentation Models & Shell Conformance](09%2027%20%E2%80%94%20UI-Agnostic%20Core,%20Ports%20Adapters,%20Presenta%203f19bb7d023f81bf8812d6f9f254d8ca.md)

[09.28 — Plugin SDK, Safe Extension Contract, Capability UX & Runtime-Neutral API](09%2028%20%E2%80%94%20Plugin%20SDK,%20Safe%20Extension%20Contract,%20Capab%203f19bb7d023f8158bb33fa413179e775.md)

[09.29 — MCP API Usability, Agent Contracts, Safety, Discovery & Deterministic Automation](09%2029%20%E2%80%94%20MCP%20API%20Usability,%20Agent%20Contracts,%20Safety%203f19bb7d023f81f68209def832efe901.md)

[09.30 — Plugin Runtime ADR: Trusted Python, Out-of-Process Python & Optional WASM](09%2030%20%E2%80%94%20Plugin%20Runtime%20ADR%20Trusted%20Python,%20Out-of-%203f19bb7d023f81f1b609d4a15da4359f.md)