# 05.1 — GPUI Primary UI Stack, Ecosystem & Icons

<aside>
🎯

**Canonical UI direction:** GPUI is the primary/default desktop UI target for Aubrieta Design Suite. The core remains GUI-agnostic, but implementation work should assume GPUI first unless a concrete blocker justifies an adapter experiment.

</aside>

# Architectural position

Aubrieta should not build directly on raw GPUI alone. The preferred stack is layered so the project can reuse production infrastructure while keeping its own visual identity:

```
Zed GPUI
    ↓
gpui-kit facade
    ↓
gpui-base
    ↓
Aubrieta Design System
    ↓
Aubrieta Panels / Personas / Tools
    ↓
AubrietaGuiBridge
    ↓
Actions / Commands / Application Core
```

The rule is **reuse behavior, own presentation**.

# Primary GPUI ecosystem

| Crate / project | Aubrieta role | Adoption |
| --- | --- | --- |
| GPUI | GPU-accelerated native desktop UI foundation, entities/views/elements, actions, focus, key dispatch, async executor and platform integration. | Canonical UI foundation |
| `gpui-kit` | Preferred facade: keeps matching GPUI/Kit versions aligned, re-exports the relevant layers and provides application/bootstrap helpers. | Canonical facade |
| `gpui-base` | Unstyled behavior/state/infrastructure: text editing, selection, docking, virtual lists, dialogs, popovers, controls, navigation, motion, history and related foundations. | **Primary reusable UI layer** |
| `gpui-component` | Complete styled component library built on base. Use selectively for mature components and as prior art; do not inherit its visual identity wholesale. | Selective |
| `gpui-kit-assets` | Default icon/assets package, including Lucide integration and compile-time icon selection helpers. | Default asset source |
| `gpui-fps` | FPS/frame-time/resource instrumentation for Developer Mode and performance gauntlets. | Dev-only |
| `gpui-wry` | Optional embedded WebView for help/manual/release notes or plugin documentation. | Optional; never creative canvas |
| `gpui-shell` | Capability-oriented JavaScript runtime hosted by Rust. Study only as prior art for scripting/UI-host patterns. It does not define Aubrieta's Plugin SDK or runtime choice. | Research/optional |
| GPUI Kit Chart/Plot | Histograms, curves, scopes, telemetry and data visualizations. | Prototype first |
| `gpui-plot` | Community native plotting alternative with zoom/pan and Plotters integration. | Experimental fallback |
| `gpui-ui-kit` | Independent community component toolkit with menus, dialogs, cards, color picker, wizard and other widgets. | Prior art / selective reuse |
| `gpui-engram` | Small community component library and reference for additional GPUI patterns. | Prior art only |
| `embedded_gpui` | Zed research for sandboxed GPUI-in-WASM UI composition. Valuable reference for plugin panels and remote-capability design. | Research only |

# Aubrieta Design System on top of GPUI

Do not turn `gpui-component` into the product identity. Aubrieta owns its own tokens and visual contracts:

- color roles and semantic state colors;
- typography scale and dense desktop metrics;
- spacing, radii, separators and elevation;
- tool rail and context-toolbar primitives;
- tabs, panels and docking chrome;
- Layers rows, object badges and thumbnails;
- property rows/editors;
- Design/Photo persona switch;
- selection/active/focus/disabled/error states;
- animation/motion rules;
- icon sizing/alignment rules;
- accessibility semantics and keyboard focus ordering.

`gpui-base` should be preferred whenever the same behavior can be styled cleanly by Aubrieta. `gpui-component` is appropriate when its implementation is mature and adopting it does not create visual/architectural debt.

# Docking and workspace

Use GPUI Kit's pure-data docking/layout infrastructure as the first implementation for:

- nested horizontal/vertical splits;
- tab groups;
- resizable panels;
- drag/drop placement;
- workspace persistence;
- persona-specific presets;
- user presets;
- reset-to-default;
- panel visibility/actions.

Workspace state is UI configuration and must remain outside the canonical document. A document never owns which panels are visible.

# Large trees and virtualized surfaces

Use GPUI Kit Tree/List/VirtualList/DataTable foundations for Layers, Assets, History, Data Merge records and other large collections. Required gauntlets:

- 10k visible/virtualized rows baseline;
- 100k synthetic rows stress target where the underlying component supports it;
- multi-selection;
- keyboard navigation;
- drag/reparent;
- inline rename;
- expand/collapse;
- thumbnails and badges;
- stable scrolling while document deltas arrive.

# Action, shortcut and command architecture

GPUI's Actions/Key Contexts are a primary architectural inspiration, but Aubrieta remains authoritative over its command model:

```
UI / Menu / Shortcut / Command Palette / Plugin / MCP
                    ↓
                Aubrieta Action
                    ↓
                Command
                    ↓
             DocumentMutator
```

GPUI actions are adapter-level bindings to Aubrieta semantic actions. Domain commands must never depend on GPUI action structs.

# Text, IME and editors

Leverage GPUI Kit/base text-editing primitives for application UI fields, command palette, numeric/property editors, search, labels and scripting surfaces. The creative document typography engine remains Parley/HarfRust/Skrifa/Fontique/ICU4X and is not replaced by GPUI's UI text field implementation.

Tree-sitter/LSP/editor hooks from GPUI Kit are useful for future scripts, plugin editors, expressions and developer tooling, not for canonical document typography.

# Plotting and Photo analysis

Prototype built-in GPUI Kit Chart/Plot first for:

- histogram;
- Curves editor graph;
- Levels histogram;
- channel/scopes views;
- performance graphs;
- cache/resource telemetry.

Adopt `gpui-plot` only if it materially outperforms or offers functionality missing from GPUI Kit. Creative graph editors may eventually require Aubrieta-specific canvas primitives rather than a generic plotting library.

# Icon architecture

Icons are part of the Aubrieta design system, not part of the domain model.

## Semantic registry

Define a toolkit-independent registry such as:

```
IconId
├── Move
├── NodeEdit
├── Pen
├── BooleanUnion
├── BooleanSubtract
├── StrokeAlignInside
├── Mask
├── AdjustmentCurves
├── ChannelCyan
├── DataMerge
└── ...
```

Actions, panels and tools refer to `IconId`; the GPUI adapter resolves the actual SVG/component.

## Primary families

### Lucide — default baseline

GPUI Kit's default `assets` feature already bundles **Lucide** through `gpui-kit-assets`. Use it as the initial general-purpose family because it requires the least glue and remains aligned with GPUI Kit. Its assets can be selected explicitly at compile time.

### Phosphor — official optional Aubrieta family

Use **`gpui-phosphor`** as the first optional icon-family adapter. It embeds the Phosphor catalog, generates typed icon names, supports GPUI SVG rendering and exposes multiple weights: Regular, Thin, Light, Bold, Fill and Duotone. This is especially attractive for a creative application because weight/fill variants can communicate active tool state cleanly.

### Aubrieta custom SVG library — mandatory

Generic icon packs will not cover professional creative-tool semantics precisely enough. Maintain an internal SVG family for:

- Pen/Node/Corner/Knife/Shape Builder variants;
- boolean/pathfinder operations;
- stroke alignment/caps/joins;
- masks/clipping/alpha locks;
- channels and CMYK-specific controls;
- gradients/mesh/perspective/warp;
- raster adjustments and Photo tools;
- text frame/layout/data-merge operations;
- export/preflight-specific glyphs.

Custom icons should use the same optical grid, stroke policy and bounding-box rules as the selected general family.

### SF Symbols — platform-native optional layer

`gpui-symbols` can provide macOS SF Symbols for platform-native affordances. Use only for OS-level chrome/actions where platform adaptation is desirable; never use SF Symbols as the canonical cross-platform representation of Aubrieta tools.

### Other SVG families

Tabler, Iconoir or other permissively licensed sets may be supported later through a generic SVG asset adapter. GPUI does not require a dedicated crate merely to render such assets. Any new family must map into the same semantic `IconId` registry.

## Icon fallback policy

```
Aubrieta custom icon
    ↓ if absent
Active general family (Lucide / Phosphor / future)
    ↓ if absent
Lucide baseline fallback
```

## Icon quality requirements

- consistent 16/20/24 px optical grids where applicable;
- theme tint through current/semantic color, not baked arbitrary colors;
- active/selected state distinct without relying only on color;
- high-DPI crisp SVG rendering;
- accessible labels from the action/tool semantics, never from filename;
- licenses and attribution tracked in THIRD_PARTY_NOTICES.

# File dialogs, clipboard and OS integration

Prefer GPUI/GPUI Kit platform services when they satisfy requirements. Keep thin adapters for OS-dependent services so fallbacks such as `rfd`/platform-specific implementations can be introduced without leaking into tools or document code. Clipboard, opening URLs, drag/drop and notifications follow the same rule.

# WebView and embedded documentation

`gpui-wry` is optional for online/manual/help surfaces. Rules:

- no document rendering through WebView;
- no essential editing functionality dependent on HTML/JS;
- no plugin receives unrestricted WebView/native bridge by default;
- external content has explicit navigation/security policy.

# Extensions

Canonical extension architecture is the **runtime-neutral semantic Plugin SDK + capability broker** defined in 09.28. **Lua 5.5 via `mlua` is the accepted V1 scripting runtime**; Wasmtime/WASI is the preferred high-isolation component tier. Python/JavaScript remain external automation languages through MCP/client SDKs unless a later ADR adds another embedded runtime. Study `gpui-shell` and `embedded_gpui` for:

- capability grants;
- UI panel extensibility;
- sandboxed script/guest views;
- safe host/guest data flow;
- keeping layout/paint paths native Rust.

Any **Post-V1 Candidate** multi-tier expansion is allowed only if every tier reuses the same semantic Actions/Commands/properties/contribution schemas. Do not create a JavaScript-only, Python-only or WASM-only parallel business-logic API.

# Testing, accessibility and profiling

Use GPUI upstream `test-support` / `#[gpui::test]` where compatible and GPUI Kit's accessibility-driven testing APIs. Required layers:

- unit/state tests outside the visual tree;
- action/key-context tests;
- keyboard/focus traversal;
- mouse/drag/drop tests;
- IME/text input tests;
- accessibility semantic-tree tests;
- docking persistence/restore;
- large-list/tree virtualization stress;
- high-DPI matrix;
- visual regression/golden screenshots;
- performance budgets with `gpui-fps` and GPUI profiling/bench features in dedicated builds.

# Developer Mode

Create a Aubrieta Developer Mode built around GPUI panels and optional `gpui-fps`:

- FPS/frame-time/P95;
- CPU/GPU/memory telemetry where available;
- render cache inspector;
- raster tile-cache inspector;
- document dirty/invalidation graph;
- action/event inspector;
- focus/key-context inspector;
- semantic/accessibility tree;
- active theme/token viewer;
- icon browser;
- plugin capability inspector;
- MCP/automation inspector.

# Dependency policy

1. Prefer `gpui-kit` facade over independently pinning mismatched GPUI ecosystem versions.
2. Pin compatible GPUI/Kit releases or immutable commits according to project stability policy; never silently follow `main` for release builds.
3. Keep community crates optional behind feature flags until maintenance and compatibility are proven.
4. Do not duplicate a capability already robustly provided by `gpui-base`/`gpui-component` without a measured reason.
5. Do not import a large UI toolkit merely for one small widget; adapt or implement the minimal component instead.
6. Third-party UI libraries must not leak into domain/public core APIs.

# Immediate implementation sequence

1. Bootstrap Aubrieta shell using `gpui-kit`.
2. Build a minimal Aubrieta Design System over `gpui-base`.
3. Implement application frame: title/menu area, persona switch, tool rail, document tabs, canvas host, right-side dock, status bar.
4. Implement Layers Tree, Properties and Color using GPUI Kit/base primitives.
5. Add Action Registry ↔ GPUI action/key-context adapter and command palette.
6. Add Lucide baseline via `gpui-kit-assets` and semantic `IconId` registry.
7. Add `gpui-phosphor` adapter and Aubrieta custom SVG asset source.
8. Add workspace serialization/docking presets.
9. Add GPUI test-support, accessibility gauntlets, visual regression and `gpui-fps` developer instrumentation.
10. Only then introduce optional WebView, scripting or community crates that solve a proven need.

# Out of scope for this page

3D/OpenGL viewport research is deliberately excluded from Aubrieta Design Suite documentation. That investigation belonged to a separate curiosity/project context and must not influence the GUI decision or Aubrieta roadmap.

# Canonical interface specification

The detailed UI/UX implementation contract now lives in [08 — Interface Atlas & Aubrieta Design System](08%20%E2%80%94%20Interface%20Atlas%20&%20Aubrieta%20Design%20System%203db9bb7d023f816898b8cdc5efef3c76.md). This GPUI ecosystem page defines **technology choices and reusable infrastructure**; the Interface Atlas defines appearance, anatomy, interaction, controls, panels, dialogs, windows, accessibility and quality gates. When the two overlap, technology must adapt to the Atlas unless an ADR explicitly changes the product specification.