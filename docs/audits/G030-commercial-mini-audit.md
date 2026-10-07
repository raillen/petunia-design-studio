# G030 — Commercial-Mini Audit Report

Gate: **First Integrated Commercial-Mini Workflow** (depends G028, G029).
Auditor role: release-verifier + tester + UX/accessibility/performance reviewer.
Date: 2026-10-06. Scope: Python application (PySide6/Qt Quick), PTND v1, C++ core baseline.

## Decision

**GO** — the architecture has crossed from framework scaffold to a credible
design-editor foundation. Foundational gaps found are non-blocking and are
tracked as explicit G059/G048 items (see criteria table). Broad feature
expansion (text/layout wave, then raster) may proceed.

## Scenario evidence

Composition: "Aurora — Social Card", 1080×1080 (social/brand card).
Built with the real product stack: surface (document canvas), parametric
shapes (rectangle wash + gradient, two ellipses), Pen path (four-petal
bloom mark, smooth bezier handles, tips snapped to the 8-unit grid),
fills/gradients/strokes on every visible object, layer rename + group
("Accents"), PTND save/reopen, PNG + SVG export.

Artifacts (bundle): `benchmarks/output/`
- `commercial-mini.ptnd` — saved document package
- `commercial-mini.png` — rasterized card (53 KB)
- `commercial-mini.svg` — vector card (1.7 KB, contains `<path>`)
- `audit-evidence.json` — machine-readable check results

## Criteria assessment

| # | Criterion | Verdict | Evidence |
|---|-----------|---------|----------|
| 1 | No P0 correctness issues | PASS | 66/66 Python tests; QML offscreen smoke clean; C++ smoke + ctest green |
| 2 | No data-loss on save/reopen | PASS | semantic snapshot before save == reopen (`roundtrip_semantic_equal: true`) |
| 3 | No known mutation bypass | PASS | static audit: zero direct `doc.objects` mutations outside `model.py` commands; bridge only reads the document |
| 4 | Save/recovery fault suite baseline | PASS (baseline) | atomic package write (tmp + verify + `os.replace`); roundtrip suite passes. Fault-injection/crash-recovery is G048 (future, not part of this gate's baseline) |
| 5 | Critical UI accessible | PASS (baseline) / GAP (AT) | full keyboard flow (V/R/O/P/N tools, Ctrl+Z/Y, Delete, Ctrl+G, Enter/Escape, snap grid). Screen-reader/AT conformance is G059 — documented gap, not P0 for this gate |
| 6 | Renderer stable across supported test OS | PASS (single OS) | offscreen Linux render stable; Windows/macOS matrix is G059 — single-OS evidence, justified gap |
| 7 | Performance within/near budgets | PASS | all provisional budgets met: boolean 10k union 0.16s (budget 2.0s), flatten 100k 0.05s (1.0s), SVG 0.1ms (0.5s), PNG 50ms (2.0s), PTND roundtrip 0.7ms (1.0s) — `benchmarks/results.json` |
| 8 | Docs match behavior | PASS | README layout/bootstrap updated to match implemented modules (paths, boolean, stubs, pyclipper dep) |

## Capability matrix (milestone status)

| Wave | Milestones | Status |
|------|-----------|--------|
| Foundation | G001–G027 (ids, math, error, document, command, jobs, bindings, QML shell, model, history, snapping, groups, SVG/PNG export, PTND package) | DONE |
| Golden workflow | G028 (groups, multi-select, keyboard, gradients, semantic roundtrip) | DONE |
| Vector geometry | G029 (canonical VectorPath/Contour/Node, cubic math, topology ops, pen/node tools) | DONE |
| Audit gate | G030 (this report) | DONE — GO |
| Vector breadth | G031 (boolean engine, compound paths, live boolean, Shape Builder, provenance, fuzz corpus) | DONE |
| Vector breadth | G032 (symbols/instances/overrides, styles, asset library) | TODO |
| Text & layout | G033–G038 (story model, shaping, line breaking, controllers, pages, PDF writer) | TODO (next wave) |
| Color | G039–G040 (ICC engine, CMYK/spot, soft proof) | TODO |
| Raster/photo | G041–G047 (tiles, image decode, brushes, selection, adjustments, retouch, histogram) | TODO |
| Durability/IO | G048–G050 (autosave/recovery, TIFF/WebP, resource manager) | TODO |
| Plugins/MCP | G051–G054 (plugin host, SDK, MCP server, UI conformance) | TODO |
| Data merge | G055–G056 (providers/bindings, panel/batch) | TODO |
| Prepress | G057–G058 (separations, PDF-X) | TODO |
| Hardening/release | G059–G060 (SLOs, sanitizer/fuzz campaigns, AT/DPI matrix, packaging) | TODO |

## Architecture drift findings

1. **Python is the execution path; C++ core is a baseline.** The C++ core
   (`cpp/petunia_core`, 731 LOC: ids/math/error/document/command/jobs) is
   far behind the Python model (paths, boolean, bridge, QML). Drift is
   intentional per the milestone DAG (Python app first), but the "C++23
   core" claim in the README header now overstates parity. Mitigation:
   either port G023–G031 subsystems to the core or relabel the header to
   reflect the staged plan. Tracked for the text/layout wave planning.
2. **Boolean backend is polyline-only (pyclipper/Clipper 1.x).** Spec 09.6.4
   prefers curve-preserving output. Fidelity policy is implemented and
   documented in `boolean.py` (flatten tolerance 0.25, quantization 1/1000,
   collinear/zero-edge cleanup, orientation convention, nonzero fill
   authoritative). Contour-level provenance for binary ops is object-level
   only (requires Clipper2 PolyPath). Accepted for GO; revisit at G059.
3. **Path object frame is derived, not authoritative.** `x/y/width/height`
   of path objects are computed from node bounds; `SetPathFrameCommand`
   translates/scales geometry to match inspector edits. Consistent, but
   means path transforms are baked (no non-destructive frame transform).
   Acceptable for GO; non-destructive transforms are a natural G032+ item.
4. **Live boolean evaluation is on-demand, not reactive.** `refreshLive()`
   recomputes; there is no automatic invalidation on operand edit. Documented
   in bridge UX ("Atualizar"/Consolidar). Reactive evaluation is deferred.
5. **No autosave/journal yet.** Save is atomic and roundtrip-verified, but
   crash recovery (G048) does not exist; criterion 4 is graded "baseline"
   accordingly.

## Prioritized next wave (per gate decision)

1. **G032** — Symbols/Instances/overrides, Object/Character/Paragraph
   Styles, Asset Library (vector breadth, closes the vector story).
2. **Text & layout wave G033–G038** — story storage, Unicode/shaping
   (HarfBuzz adapter), line breaking/justification, tool controllers + IME,
   pages/surfaces/guides, PDF writer baseline. (Gate explicitly orders
   text/layout before raster.)
3. **Raster wave G041–G047** — tile store, image decode (libvips),
   brushes, selection, adjustment nodes, retouch tools.

## How to reproduce

```bash
PYTHONPATH=python .venv/bin/python tooling/commercial_mini.py   # composition + evidence
PYTHONPATH=python .venv/bin/python benchmarks/run_benchmarks.py  # benchmark bundle
.venv/bin/pytest -q                                              # correctness
.venv/bin/ruff check python tests tooling && .venv/bin/pyright   # lint/typecheck
PYTHONPATH=python QT_QPA_PLATFORM=offscreen .venv/bin/python -m petunia_app --smoke
```
