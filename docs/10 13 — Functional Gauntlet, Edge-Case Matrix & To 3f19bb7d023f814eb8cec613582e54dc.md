# 10.13 — Functional Gauntlet, Edge-Case Matrix & Tool Definition of Done

# Definition of Done for a tool

A tool is complete only when all applicable items pass:

- ToolId/ActionIds registered;
- icon/TextId/help/shortcut;
- pointer/pen/key state machine;
- context toolbar schema;
- Properties schema;
- modifiers documented;
- preview/commit/cancel;
- one coherent undo transaction;
- core Command tests;
- save/load roundtrip;
- MCP/plugin discoverability or explicit exclusion;
- accessibility keyboard alternative;
- mixed DPI/zoom behavior;
- performance budget;
- error/disabled reasons;
- visual/semantic evidence.

# Cross-tool gauntlet

For every tool run:

1. empty document;
2. single eligible object;
3. multi-selection;
4. locked object;
5. hidden object;
6. nested transformed group;
7. clipped/masked object;
8. symbol instance;
9. multiple Surfaces;
10. extreme zoom in/out;
11. alternate units;
12. high DPI;
13. undo/redo repeated;
14. save/reopen after result;
15. cancel mid-gesture;
16. switch Persona mid-safe state or block correctly;
17. close document/window during staged work;
18. plugin providing related panel absent;
19. automation equivalent path;
20. malformed/extreme numeric input.

# Geometry edge cases

Zero-length segments, coincident nodes, self intersections, tiny/huge coordinates, singular transforms, near-parallel booleans.

# Raster edge cases

Stroke at tile boundaries, sparse tiles, huge image, alpha-only, mask target, 16-bit/float, memory pressure, GPU lost.

# Text edge cases

Empty text, overset, bidi, complex scripts, emoji/ZWJ, IME, missing fonts, variable axes, text-on-path degenerate.

# IO edge cases

Disk full, permission lost, destination changed, path too long, Unicode filename, interrupted atomic save, corrupt package/resource.

# Plugin/MCP edge cases

Permission revoked, process crash, stale revision, oversized response, cancel job, unavailable contribution, version mismatch.

# Performance budgets

Set per target hardware with baseline fixtures. No arbitrary "fast" claim. Key interactions target pointer-to-preview within one frame under normal fixture; complex final operations may background with progress/cancel.

# Evidence bundle

Revision, fixture IDs, tool manifest, commands/tests, screenshots/semantic snapshot, benchmarks where relevant, known limitations and reviewer verdict.

# Parity ledger

Each old Petunia Rust/Slint functional page maps to this atlas. Any intentional scope change receives ADR/status, never silent omission.