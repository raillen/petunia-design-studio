# 08.15 — Slint Implementation Map & Component Ownership

<aside>
🧩

**Canonical implementation map — 2026-09-21:** Slint is the primary Petunia Design Studio shell. This page supersedes the former GPUI map. The core remains GUI-agnostic and every visible feature crosses the semantic bridge rather than reaching into document internals.

</aside>

# Layer map

| Area | Primary implementation | Ownership |
| --- | --- | --- |
| App shell/windows | Slint | UI adapter |
| Design tokens/themes | Petunia Design System | Petunia |
| Semantic UI models | PetuniaDesignGuiBridge DTOs/deltas | Petunia |
| Common controls | Slint primitives wrapped by semantic Petunia components | Shared/Petunia |
| Docking/workspaces | Petunia docking model rendered in Slint | Petunia |
| Layers/Assets/History | Slint models/ListView + virtualization adapters | Petunia semantics |
| Command palette | Slint surface + ActionRegistry | Petunia |
| Text fields/IME | Slint/platform text input + Petunia property editors | Shared/Petunia |
| Document typography | Parley/HarfRust/Skrifa stack | Core, not Slint |
| Color editing UI | Slint components + petunia_design_color semantics | Petunia |
| Icons | Lucide/Tabler sources + custom SVG behind IconId | Petunia registry |
| File/folder dialogs | Platform adapter, rfd/native provider when validated | Adapter |
| Accessibility | Slint/platform semantics + explicit Petunia metadata | Shared/Petunia |
| Canvas renderer | Existing scene/render/compositor engines | Core/engine |
| Viewport host/input | Slint shell + normalized semantic input | Adapter |
| Performance HUD | Petunia diagnostics, dev-only | Petunia |
| Plugins | runtime-neutral SDK; Lua/WASI adapters | Extension host |

# Slint source ownership

Recommended layout:

```
ui/
├── app_window.slint
├── tokens/
├── themes/
├── primitives/
├── components/
├── shell/
│   ├── menu_bar.slint
│   ├── persona_bar.slint
│   ├── context_toolbar.slint
│   ├── document_tabs.slint
│   ├── tool_rail.slint
│   ├── dock_host.slint
│   └── status_bar.slint
├── panels/
├── dialogs/
├── canvas/
├── design/
├── photo/
├── accessibility/
└── dev_gallery/
```

Rust-side adapter modules own model conversion, callback wiring, platform requests, input normalization and renderer-host lifecycle.

# PetuniaDesignGuiBridge

The bridge exposes coarse semantic state only:

- application/session snapshot;
- document tabs/session metadata;
- ActionState map;
- selection summary;
- typed property schemas/values;
- panel presentation models/deltas;
- tasks/progress;
- semantic notifications/dialog requests;
- viewport session handles;
- TextId/IconId/HelpTopicId;
- inspection/test metadata.

It does **not** expose mutable DocumentStore pointers, Slint types, raw renderer objects, third-party geometry structs or toolkit callbacks.

# Component ownership

Canonical components: PetuniaButton, PetuniaIconButton, ToolButton, SegmentedControl, SearchField, TextField, NumericField, SliderField, ComboBox, PropertyRow, ColorSwatch, ColorPickerShell, PopupMenu, MenuRow, Tooltip, Panel, PanelTab, PanelSection, TreeRow, Splitter, DockTargetOverlay, ModalShell, Popover, Toast, EmptyState, InlineDiagnostic, ProgressRow and TabStrip.

Each component owns presentation/interaction semantics, not domain behavior.

# Required component states

Default, hover, pressed, keyboard focus, disabled, selected/toggled, mixed/indeterminate, loading/busy, error, warning, long/localized copy, Dark/Light, Compact/Comfortable/Spacious and 100/150/200% scale.

# Design-system discipline

Production feature UI cannot hard-code user-visible strings, raw colors, icon filenames, arbitrary font metrics, spacing, radii, motion durations or focus treatments. It consumes TextId/IconId/TokenId and canonical components.

# Models

Large collections use incremental/virtualized Slint models. Stable semantic IDs identify rows; list index is never persistent identity. Hover/focus must not rebuild whole models. Async deltas carry revision identity so stale results can be discarded.

# Docking implementation

The **workspace model is Petunia-owned**, not a Slint widget tree serialized ad hoc. It stores dock nodes, split ratios, panel instances, tab order, floating geometry, visibility/collapse state and provider identity. Slint renders this model and emits semantic layout actions.

Required behavior: nested left/right/bottom splits, tab reorder, panel move, split preview, min/preferred sizes, float/redock where platform support is accepted, Esc cancel, keyboard move commands, restart/restore and missing-provider recovery.

# Canvas boundary

Slint hosts window geometry, viewport bounds, focus and normalized input. Scene extraction, vector/raster evaluation, compositing, color transforms and canonical selection semantics remain outside Slint. Renderer failure/device loss produces a semantic diagnostic/recovery path instead of an empty canvas.

# Input

Pointer/key/pen events are normalized before tool logic. Capture is released on commit, cancel, focus loss and interruption. IME composition suppresses global shortcuts. Pen pressure/tilt/eraser identity are optional semantic fields, never fabricated.

# Native services

Open/Save/Choose Folder, clipboard, drag/drop, URL help, notifications, monitor/DPI and font enumeration live behind typed platform ports. Slint feature code requests these services through application/platform adapters rather than direct OS calls.

# Accessibility

Every interactive component has role, semantic name, current value/state, enabled/disabled, focusability, actions and stable test/accessibility ID. If Slint/platform support cannot expose a required semantic, the limitation is tracked explicitly and can block the applicable release profile.

# Icon registry

Provider order:

1. Petunia custom domain icon;
2. selected validated icon family;
3. canonical Lucide/Tabler fallback;
4. safe missing-icon diagnostic.

General UI glyphs are normalized to consistent optical size/stroke. Creative-tool icons such as Pen, Node, Contour, Corner, Boolean, Stroke Width, Surface, Gradient and Transparency use purpose-designed assets where library icons are ambiguous.

# Dev gallery

Maintain a developer-only component gallery containing all controls, states, themes, densities, locales, RTL sample, long labels, keyboard focus, accessibility metadata, icon browser and stress datasets. This is a canonical visual-regression surface.

# Performance constraints

Re-baseline after Slint migration: release startup/time-to-interactive, clean idle CPU/RAM, large Layers/Assets/Fonts models, docking drag, panel resize while canvas renders, command palette, text/IME, Persona switching, theme/DPI changes and pointer-to-feedback latency.

Do not inherit GPUI benchmark claims.

# Security constraints

Slint is presentation, not trust. Numeric/text requests are revalidated in application/domain. Clipboard/drop/resource packs/imported assets and PTND data are hostile inputs. UI callbacks never bypass capability permissions, command validation or path/file safety.

# Shell conformance

Slint and MockGuiAdapter must pass shared semantic tests for actions, properties, selection, jobs, dialogs, document lifecycle and presentation state. Slint adds toolkit-specific focus, DPI, accessibility, keyboard, pointer, visual and renderer-host tests.

# Definition of Done

This adapter is complete only when representative Design + Photo workflows run through the public semantic bridge; no domain crate imports Slint; no visible V1 control is dead; workspace persistence survives restart; localization/DPI/accessibility matrices pass; and current performance/interaction evidence exists for the final revision.