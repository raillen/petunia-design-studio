# 08.15 — GPUI / GPUI Kit Implementation Map & Component Ownership

# Rule

GPUI is primary implementation target. Core remains GUI-agnostic. This page maps UI concepts to likely GPUI ecosystem layers and identifies where Aubrieta must own the component.

| Area | Primary implementation | Ownership |
| --- | --- | --- |
| App/window/entity/action foundation | GPUI | Framework |
| Facade/version alignment | `gpui-kit` | Framework |
| Focus/selection/overlay/virtualization primitives | `gpui-base` | Reuse behavior |
| Common mature controls | `gpui-component` selective | Reuse/adapt |
| Visual tokens/components | Aubrieta Design System | **Aubrieta** |
| Dock layout | GPUI Kit/base docking | Reuse + Aubrieta chrome |
| Layers tree | Tree/VirtualList foundation | Aubrieta row/drag semantics |
| Data Merge grid | DataTable/virtualization | Aubrieta data model |
| Command palette | GPUI/Kit UI + Aubrieta Action Registry | Shared |
| Text fields/IME in shell | GPUI Kit/base | Framework behavior |
| Document typography | Parley stack | Core, not GPUI |
| Histogram/plots | GPUI Kit Chart/Plot first | Prototype/adapt |
| Color picker shell | GPUI base/component + `aubrieta_color` adapter | Aubrieta semantics |
| Icons | `gpui-kit-assets` Lucide + `gpui-phosphor`  • custom SVG | Semantic registry Aubrieta |
| Performance HUD | `gpui-fps`  • Aubrieta diagnostics | Dev only |
| Help WebView | `gpui-wry` optional | Optional adapter |
| Plugin scripting adapter | Lua 5.5 via `mlua`, behind the runtime-neutral Plugin SDK; `gpui-shell`/`embedded_gpui` remain prior art only | Aubrieta extension host; **not owned by GPUI UI layer** |

# Aubrieta UI crate boundaries

Recommended conceptual modules:

```
aubrieta_ui_gpui
├── app_shell
├── design_system
│   ├── tokens
│   ├── icons
│   ├── typography
│   ├── controls
│   └── motion
├── docking
├── menus
├── command_palette
├── canvas_host
├── panels
│   ├── layers
│   ├── properties
│   ├── color
│   ├── assets
│   ├── history
│   ├── typography
│   ├── photo
│   └── data_merge
├── dialogs
│   ├── new_document
│   ├── import
│   ├── export
│   ├── preferences
│   └── recovery
├── accessibility
├── workspace_state
└── testing
```

Do not physically split every module into a crate until compile-time/team boundaries justify it.

# AubrietaGuiBridge

Coarse-grained interface exposes:

- application/session snapshots;
- selection/property view models;
- action state map;
- panel data models/deltas;
- background task state;
- dialogs requests;
- canvas viewport/session handles;
- localized strings/help topic IDs.

Do not pass Kurbo/Vello/wgpu/document internals through ordinary panel view models. The same bridge contracts must be consumable by a headless/mock shell, and feature code may not call `gpui` APIs through hidden global singletons/service locators.

# Design System implementation

Use `gpui-base` when possible for behavior. Wrap components in Aubrieta types such as:

`AubrietaButton`, `AubrietaIconButton`, `PropertyField`, `PanelSection`, `AubrietaTreeRow`, `ToolButton`, `ContextToolbarGroup`, `AubrietaPopover`, `AubrietaDialog`.

This ensures token updates and accessibility fixes propagate across the suite.

# Icon registry

`IconId` resolves through provider priority:

1. Aubrieta custom;
2. active user icon family;
3. Lucide baseline fallback.

Provider returns SVG/vector asset compatible with GPUI. Action registry supplies accessible label independently.

# Styling discipline

No feature panel should hardcode literal colors/radii/spacing except prototype code. CI/lint or code-review policy should flag ad-hoc styling. Story/gallery app displays every Aubrieta component in all states/themes/densities.

# Component gallery

Maintain executable `aubrieta-ui-story` analogous to GPUI Kit story:

- controls;
- states;
- theme/density matrix;
- long labels/locales;
- keyboard focus;
- accessibility inspector;
- stress scenarios;
- icon browser.