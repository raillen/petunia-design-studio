# 17.3 — Milestone Task DAG from Empty Repository to Commercial V1

# Wave 0 — Control plane

Prumo pin, repository policy, docs router, CMake/pyproject skeleton, CI, diagnostics and coding standards.

# Wave 1 — Kernel

Parallel where independent:

- typed IDs/value types;
- DocumentStore hierarchy;
- ActionRegistry/Command/Transaction;
- JobScheduler skeleton;
- schema tooling.

Then integrate history + semantic snapshot.

# Wave 2 — PTND minimal

Manifest/document JSON, resource index, atomic save, load/validate, CLI validator, golden fixtures.

# Wave 3 — Qt shell

Design tokens, MainWindow, QAction adapter, dock model, document tabs, generic Property editor, semantic UI inspection.

# Wave 4 — Renderer spike

RenderScene minimal rectangles/images/text placeholders; candidate backends measured. ADR selects backend before broad rendering work.

# Wave 5 — Vector vertical slice

Rectangle/ellipse, Move/Transform, hit test, snapping, fill/stroke, Layers/Properties, SVG/PNG export.

# Wave 6 — Path vertical slice

Pen/Node, cubic paths, geometry, stroke rendering, booleans, undo/save/export.

# Wave 7 — Text/layout

Text engine, caret/IME, typography panels, Surface/pages/guides/margins, PDF output baseline.

# Wave 8 — Raster vertical slice

TileStore, image place/decode, brush, pixel selection/mask, compositor, adjustment baseline.

# Wave 9 — Professional IO/color

ICC, CMYK, proofing, TIFF/PDF maturation, preflight, resource manager, recovery.

# Wave 10 — Extensibility

Plugin semantic SDK/host, MCP methods/inspection, examples and security verification.

# Wave 11 — Product breadth

Shape Builder, retouch suite, Data Merge, richer effects, symbols/styles/assets, workspace polish.

# Wave 12 — Commercial hardening

Long sessions, performance budgets, accessibility matrix, fuzzing, installers, updater, SBOM/license, support diagnostics.

# Gate discipline

Downstream feature wave can prototype early but cannot redefine upstream contract after gate without ADR/migration.