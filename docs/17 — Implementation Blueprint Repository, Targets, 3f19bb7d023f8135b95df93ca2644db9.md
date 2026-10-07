# 17 — Implementation Blueprint: Repository, Targets, Ownership, Milestones & First Execution Plan

# Purpose

Transformar o caderno em um plano executável de implementação. Esta página liga arquitetura, funcionalidades, GUI, workforce e evidence gates a uma topologia concreta de repositório e uma sequência de trabalho.

# Canonical repository blueprint

```
petunia-design-studio/
  apps/
    petunia_studio/
    petunia_cli/
    petunia_mcp/
    petunia_plugin_host/
  python/
    petunia_app/
      application/
      actions/
      tools/
      panels/
      dialogs/
      workspace/
      models/
      design_system/
      qt/
      platform/
      plugins/
      mcp/
  cpp/
    core/
    geometry/
    raster/
    text/
    color/
    render/
    io/
    jobs/
    application/
  bindings/
    python/
  schemas/
    ptnd/
    actions/
    properties/
    plugins/
    mcp/
  resources/
    icons/
    cursors/
    themes/
    strings/
    brushes/
  shaders/
  plugins/
    sdk/
    examples/
  tests/
    cpp/
    python/
    bindings/
    integration/
    ui/
    visual/
    accessibility/
    security/
  fixtures/
    documents/
    geometry/
    raster/
    text/
    color/
    formats/
    corrupt/
  benchmarks/
  fuzz/
  tooling/
    prumo/
    scripts/
  docs/
    adr/
    api/
    format/
    contributor/
    evidence/
```

# Ownership rule

Each top-level implementation area has one semantic owner and can have many contributors. Cross-layer changes require explicit impact map.

# Development order

Kernel first; shell second; one vertical vector slice; one vertical raster slice; text/color/IO; then breadth. Avoid building dozens of disconnected widgets before a saved, rendered, undoable document loop exists.

# First usable vertical slice

Launch app -> New Document -> create Surface -> Rectangle tool -> set fill -> Move/resize -> Layers/Properties reflect state -> Undo/Redo -> Save .PTND -> close/reopen -> Export PNG/SVG.

This slice proves:

- Qt shell;
- Action/Command path;
- binding;
- document model;
- renderer;
- generic properties;
- serialization;
- export;
- history;
- UI tests.

# Second vertical slice

Open/place raster -> Brush stroke -> Selection -> Mask -> Adjustment -> Save/reopen -> export.

# Rule

A broad scaffold without working vertical slices does not count as meaningful product progress.

[17.1 — Target Graph, Library Boundaries & Link/Import Rules](17%201%20%E2%80%94%20Target%20Graph,%20Library%20Boundaries%20&%20Link%20Imp%203f19bb7d023f813289a7fd34a1335c95.md)

[17.2 — Subsystem Ownership Matrix: Code, Agents, Skills, Tests & Evidence](17%202%20%E2%80%94%20Subsystem%20Ownership%20Matrix%20Code,%20Agents,%20Sk%203f19bb7d023f81148c51c74b4873a9ea.md)

[17.3 — Milestone Task DAG from Empty Repository to Commercial V1](17%203%20%E2%80%94%20Milestone%20Task%20DAG%20from%20Empty%20Repository%20to%203f19bb7d023f81119b28e20f7fc7a368.md)

[17.4 — First 30 Executable Goals for Prumo / Code Agents](17%204%20%E2%80%94%20First%2030%20Executable%20Goals%20for%20Prumo%20Code%20Ag%203f19bb7d023f81d4a66fe2feef5797a1.md)

[17.5 — Vertical Slice Acceptance Tests & First Commercial Demo Definition](17%205%20%E2%80%94%20Vertical%20Slice%20Acceptance%20Tests%20&%20First%20Com%203f19bb7d023f815fbbd6c602a334b7ef.md)

[17.6 — Goals G031–G060: From Vector Core to Commercial V1](17%206%20%E2%80%94%20Goals%20G031%E2%80%93G060%20From%20Vector%20Core%20to%20Commerc%203f19bb7d023f810a8acec924076867bb.md)