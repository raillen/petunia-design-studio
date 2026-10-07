# 15 — Python/C++ Migration & Total Assurance Program

# Objective

Substituir integralmente Rust/Slint por Python/PySide6+C++23 preservando a semântica do produto e melhorando extensibilidade e velocidade de desenvolvimento.

# Phases

## P0 — Authority freeze

Congelar .PTND, Design+Photo, Surface, Action/Command, semantic IDs e parity ledger.

## P1 — Build skeleton

CMake/Ninja + pyproject/uv + nanobind + CI + lint/typecheck/sanitizers.

## P2 — Canonical core

Document, IDs, commands, transactions, history, serialization e headless tests.

## P3 — Qt shell

Main window, dock model, tabs, actions, command palette, panels, canvas host, preferences.

## P4 — Vector pipeline

Geometry, hit testing, snapping, render projection e Design baseline.

## P5 — Raster pipeline

Tiles, brush, selection/mask, compositor, Photo baseline.

## P6 — Text/color/IO

HarfBuzz/FreeType, ICC, .PTND production, import/export/preflight.

## P7 — Extensibility

Plugin host, SDK, capability broker, MCP server e semantic inspection.

## P8 — Total Assurance

Fuzzing, sanitizers, benchmarks, long-session, visual/accessibility, recovery, hostile inputs.

## P9 — Legacy cleanup

Remover autoridade Slint/Rust e adapters temporários somente após parity/evidence gates.

# Hard gates

- no Qt/Python dependency in C++ domain;
- no document mutation outside Command;
- no untyped public Python boundary;
- no third-party in-process Python plugin by default;
- no release with sanitizer/fuzz/save-recovery critical failures;
- no “Affinity parity” claim sem coverage ledger explícito.

[15.1 — Legacy Rust/Slint → Python/C++ Replacement Matrix](15%201%20%E2%80%94%20Legacy%20Rust%20Slint%20%E2%86%92%20Python%20C++%20Replacement%20%203f19bb7d023f81b0855cf0fa84444db5.md)

[15.2 — Implementation Milestones, Dependency DAG & Commercial MVP Gates](15%202%20%E2%80%94%20Implementation%20Milestones,%20Dependency%20DAG%20&%203f19bb7d023f81e09ebfcad5236abd3f.md)

[15.3 — Renderer Technology Falsification Spike & Decision Gate](15%203%20%E2%80%94%20Renderer%20Technology%20Falsification%20Spike%20&%20D%203f19bb7d023f81ea9e81f1744fd52b42.md)

[15.4 — Migration Acceptance, No-Rust/No-Slint Cleanup & Parity Exit Criteria](15%204%20%E2%80%94%20Migration%20Acceptance,%20No-Rust%20No-Slint%20Clea%203f19bb7d023f81fab8ffc8e65d5db075.md)

[15.5 — Legacy Code/Module Mapping, Preserve/Rebuild/Drop Decisions & Oracle Use](15%205%20%E2%80%94%20Legacy%20Code%20Module%20Mapping,%20Preserve%20Rebuil%203f19bb7d023f811b976fe5495e220265.md)

[15.6 — Document/File Compatibility, Legacy PTND Migration & Golden Corpus](15%206%20%E2%80%94%20Document%20File%20Compatibility,%20Legacy%20PTND%20Mi%203f19bb7d023f81b2a2f7d2d731ba5dbd.md)

[15.7 — Incremental Cutover, Feature Flags, Parallel Oracles, Rollback & Branch Strategy](15%207%20%E2%80%94%20Incremental%20Cutover,%20Feature%20Flags,%20Paralle%203f19bb7d023f813989fcc5e9cc102d38.md)

[15.8 — Migration Parity Scoreboard, Exit Evidence & Residual-Risk Ledger](15%208%20%E2%80%94%20Migration%20Parity%20Scoreboard,%20Exit%20Evidence%20%203f19bb7d023f819da648cc153355a6c4.md)

[15.5 — Migration Compatibility Harness, Legacy Fixtures & Semantic Diff](15%205%20%E2%80%94%20Migration%20Compatibility%20Harness,%20Legacy%20Fix%203f19bb7d023f81fcb661f6c8ee6697b3.md)

[15.6 — Incremental Replacement Strategy, Strangler Boundaries & No-Dual-Truth Rule](15%206%20%E2%80%94%20Incremental%20Replacement%20Strategy,%20Strangler%203f19bb7d023f81f182eccf453b38d4bd.md)

[15.7 — Migration Risk Register & Exit Evidence by Subsystem](15%207%20%E2%80%94%20Migration%20Risk%20Register%20&%20Exit%20Evidence%20by%20%203f19bb7d023f81c08220e231c716bbe4.md)