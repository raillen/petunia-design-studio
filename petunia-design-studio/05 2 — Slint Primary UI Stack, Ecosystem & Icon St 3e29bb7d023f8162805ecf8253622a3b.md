# 05.2 — Slint Primary UI Stack, Ecosystem & Icon Strategy

<aside>
🧩

**Canonical UI stack — 2026-09-21:** Slint is the primary Petunia Design Studio shell. This page supersedes the former GPUI-primary implementation path while preserving the UI-agnostic core boundary.

</aside>

# Stack

- Slint — declarative desktop shell/components;
- Rust adapter — PetuniaDesignGuiBridge binding, models, callbacks, input normalization;
- Petunia Design System — tokens, components, density, themes, interaction states;
- Lucide/Tabler — general icon source families behind semantic IconId;
- custom Petunia SVGs — domain-specific creative-tool icons;
- rfd or platform-native equivalent — file/folder dialogs behind platform adapter;
- AccessKit/platform semantics where supported — accessibility integration;
- existing renderer/engines — independent of Slint.

# Why Slint

Primary reasons: declarative UI ownership, component reuse, predictable design-system control, Rust integration, portability, explicit state binding and alignment with Petunia family desktop UI strategy. Slint is not assumed to be performant enough merely because it is native; large-tree, canvas-host, docking and input latency are measured.

# UI source layout

ui/tokens, themes, primitives, components, shell, menus, panels, dialogs, canvas, design, photo, accessibility and dev_gallery. app_window.slint composes, it does not contain domain logic.

# Component ownership

Petunia owns semantic wrappers/components when they enforce visual or behavior contracts. Avoid wrappers that simply rename a Slint primitive with no semantic value.

# Icon integration

General icons are normalized at build/resource-pack time. Product code requests IconId. Domain-specific icons are designed explicitly for node editing, Pen, boolean/path operations, Corner, Contour, Surface, Gradient, Transparency, Stroke Width and other ambiguous creative-tool concepts.

# Native dialogs

Slint feature code emits semantic requests. Rust platform adapter invokes native file/folder dialog provider. Cancel is a normal outcome, not an error.

# Large data

Layers, fonts, assets, history and data tables use model virtualization/incremental deltas. Stable IDs are semantic identities; list indices are presentation positions.

# Docking

Petunia owns docking/workspace state independent of toolkit. Slint renders tab stacks, splitters, target overlays, floating windows/palettes and restoration. Workspace persistence does not enter document state.

# Canvas

Slint hosts viewport surface and normalized input; document/scene/render semantics stay in engines. Rendering integration must be profiled and tested at resize, DPI changes, device loss and long sessions.

# Required benchmarks

Startup/time-to-interactive; idle RAM/CPU; 10k Layers scroll; 100k synthetic stress where meaningful; panel resize while canvas updates; tab/persona switching; command palette; typing/IME; docking drag; DPI/theme switching; canvas pointer latency.

# Required conformance

MockGuiAdapter and Slint adapter share semantic test suite. Forbidden dependency graph checks ensure Slint never enters domain/application contracts. Every visible control must be backed by SurfaceId + ActionId/ToolId/PropertyEdit or explicit disabled state.

# Migration from GPUI

Do not port toolkit concepts mechanically. Preserve semantic contracts, not GPUI object structure. Any GPUI-only behavior that was valuable must first be expressed as a toolkit-neutral requirement, then implemented in Slint.

# Exit

Slint becomes release-authoritative only when Design/Photo clean-state workflows, docking, keyboard/focus, accessibility, localization, performance and UI evidence gates pass on the final revision.