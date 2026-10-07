# 15.2 — Implementation Milestones, Dependency DAG & Commercial MVP Gates

# M0 — Skeleton

Repo topology, CMake, pyproject/uv, nanobind hello boundary, CI, logging, config, Prumo workforce pin.

# M1 — Document kernel

IDs, document hierarchy, Action/Command, transaction/history, JSON schema, PTND minimal save/load, CLI validator. Gate: deterministic headless tests.

# M2 — Qt shell

MainWindow, tabs, ActionAdapter, dock model, workspace persistence, design tokens, accessibility baseline. Gate: open/save minimal doc and semantic UI tests.

# M3 — Vector MVP

Canvas renderer spike winner, transform/hit/snap, shapes, pen/node, fills/strokes, layers. Gate: real logo/illustration fixture editable/save/export SVG/PNG.

# M4 — Text/Layout

HarfBuzz/FreeType, artistic/frame/path text, Surfaces/guides/margins/columns/bleed. Gate: multilingual brochure fixture/PDF.

# M5 — Raster MVP

Tiles, brush/eraser, selection/masks, crop, basic adjustments/live blur, compositor. Gate: photo retouch fixture and mixed vector+raster doc.

# M6 — Professional color/IO

ICC/CMYK/soft proof, TIFF/PDF, preflight, resource manager, robust PTND/recovery. Gate: print-oriented fixtures.

# M7 — Extensibility

Plugin host, SDK/examples, MCP discovery/mutation/jobs/inspection. Gate: third-party sample plugin + MCP cookbook parity.

# M8 — Commercial hardening

Perf, hostile input, fuzz/sanitizers, long session, accessibility, installers/update/SBOM. Gate: release verifier no critical blockers.

# Dependency rule

Do not postpone architecture boundaries (Commands, IDs, format, core/UI) until after features. Feature breadth grows only after kernel contracts have evidence.