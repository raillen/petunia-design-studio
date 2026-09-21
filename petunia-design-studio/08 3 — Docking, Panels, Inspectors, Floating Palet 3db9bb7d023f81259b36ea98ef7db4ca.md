# 08.3 — Docking, Panels, Inspectors, Floating Palettes & Workspace Presets

# Amendment 2026-09-21 — Slint Docking

The sentence that GPUI Kit docking is the first implementation is superseded. Petunia owns the docking/workspace model and implements it through Slint. Dock tree serialization, PanelId resolution, splitters, drag targets, floating palettes, missing-provider recovery and min/preferred sizes remain toolkit-neutral contracts.

# Docking model

Aubrieta uses GPUI Kit's data-oriented docking as the first implementation, wrapped by Aubrieta workspace state. Docking is **workspace state**, not document state.

Canonical regions:

- left dock zone;
- right dock zone;
- bottom dock zone;
- center document/canvas zone;
- floating palette layer;
- detached window layer.

# Panel anatomy

A standard docked panel contains:

1. tab strip when multiple panels share a container;
2. optional compact panel header;
3. optional search/filter row;
4. scrollable panel content;
5. optional persistent footer/action row.

Header actions are icon buttons with tooltips and accessible names. Avoid permanent `...` menus when 1–2 visible actions suffice; use overflow for secondary actions.

# Panel sizing

Each panel declares:

- minimum width/height;
- preferred width/height;
- maximum only when necessary;
- shrink priority;
- whether it supports horizontal/vertical layouts;
- whether it may float/detach;
- whether multiple instances are allowed.

Examples:

- Layers preferred 280–340 px;
- Properties preferred 300–360 px;
- Color 260–320 px;
- Histogram 260–360 px;
- History 260–320 px;
- Assets 320–420 px.

These are defaults, not hard constraints.

# Drag docking

When dragging a panel/tab:

- show source ghost with panel title/icon;
- highlight legal targets around container edges/center;
- use clear insertion preview for tab placement;
- show split preview before drop;
- invalid targets use forbidden cursor/state;
- dropping outside main window creates floating palette if panel allows it;
- Esc cancels drag and restores original layout;
- keyboard-accessible commands exist to move panel without drag.

# Splitters

- visible on hover/focus, otherwise subtle;
- target area wider than visible hairline;
- double-click resets to preferred ratio when appropriate;
- dragging updates live unless expensive content requires throttled preview;
- min sizes enforced without jitter;
- cursor communicates horizontal/vertical resize.

# Floating palettes

Floating palettes are nonmodal, persistent utility windows for panels such as Color, Brushes or Navigator.

Rules:

- title bar compact, with pin/dock/close controls;
- optionally always-on-top only within Aubrieta, not globally by default;
- remember size/position by workspace;
- can be redocked by drag or explicit command;
- use same panel content component as docked state;
- avoid rounded-card appearance inconsistent with desktop windows; use subtle elevated surface.

# Panel tab behavior

- reorder tabs;
- drag tab out to float/detach;
- middle-click optional close;
- context menu: Close, Close Others, Move Left/Right, Float, Dock Left/Right/Bottom, Reset Panel;
- active tab indicator has text + visual state;
- overflow menu when many tabs.

# Collapse behavior

Dock containers can collapse to:

- icon rail for rarely used panels; or
- zero width with reveal affordance, depending workspace preset.

Collapsed panel icon retains badge/attention state. Hover may preview temporarily only if not intrusive.

# Workspace presets

Built-ins:

- Design Default;
- Design Compact;
- Photo Default;
- Photo Retouch;
- Minimal Canvas;
- Dual Monitor.

User operations:

- Save Workspace As…;
- Update Current Workspace;
- Rename;
- Duplicate;
- Delete user preset;
- Reset built-in;
- Import/Export workspace layout.

Workspace stores:

- dock tree;
- panel instances and order;
- floating geometry;
- visibility;
- toolbar customization;
- density/theme override if chosen;
- Persona association;
- monitor-aware placement metadata.

# Panel content conventions

Panels use collapsible sections only when it reduces scanning cost. Do not nest collapsibles more than two levels.

Property rows generally use:

`Label | Control`

with optional reset/revert icon appearing on modification/hover.

For mixed multi-selection values, fields show mixed state (`—` or explicit `Mixed`) rather than inventing a value. Editing a mixed field applies the entered value to the selection.

# Canonical panels

Shared: Layers, Properties, Color, Swatches, Assets, History, Navigator, Transform, Align, Export.

Design-focused: Stroke, Typography, Paragraph, Styles, Symbols, Pathfinder, Data Merge.

Photo-focused: Histogram, Adjustments, Channels, Brushes, Brush Settings, Masks, Info.

Developer-only: performance, scene cache, action inspector, semantics tree.

# Modular panel-provider behavior

Panels are contributions, not hard-coded shell children. Workspace restore resolves each saved `PanelId` through the contribution registry. If a module/plugin panel is unavailable or disabled, Aubrieta must preserve the saved placement metadata where practical, omit the unavailable panel without breaking neighboring splits, show a recoverable diagnostic in Workspace/Plugin management, and restore it when the provider returns. No workspace layout may panic because a contribution disappeared.

# Tokenized metrics and preference ownership

Preferred/minimum panel sizes, splitter hit areas, tab heights and floating-palette chrome resolve through Design System/workspace metadata rather than literals in feature panels. Theme/density preference precedence is owned by 09.23; this page defines presentation behavior only. Temporary panel search/filter state is session UI state unless a panel explicitly documents persistence.