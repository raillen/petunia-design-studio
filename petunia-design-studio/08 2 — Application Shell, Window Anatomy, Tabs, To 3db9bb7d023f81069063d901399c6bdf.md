# 08.2 — Application Shell, Window Anatomy, Tabs, Toolbars & Status Bar

# Window anatomy

The primary document window is composed in this vertical order:

1. native/window frame region;
2. application menu bar integration where platform appropriate;
3. document/tab strip;
4. primary/context toolbar region;
5. main workspace with left tool rail + dock zones + canvas;
6. optional bottom panel dock;
7. status bar.

The shell must support single-window multi-document tabs as default and detached document windows as an advanced workflow.

# Title and document state

Window title communicates current document, not redundant product branding. The tab indicates:

- document name;
- modified state;
- read-only/linked/external-provider status where relevant; **cloud status exists only when an optional provider/capability contributes it** and is not a built-in document assumption;
- close button on hover/active state;
- optional pinned state;
- error/recovery indicator when autosave fails.

Unsaved documents use a readable generated label such as `Untitled 1`; never use ambiguous blank tabs.

# Document tab strip

Behavior:

- reorder by drag;
- middle-click closes when platform convention permits;
- context menu: Close, Close Others, Close Tabs to Right, Duplicate View, Reveal in File Manager, Move to New Window;
- horizontal overflow becomes scroll/overflow menu rather than shrinking tabs below legibility;
- active tab has stronger foreground contrast and subtle surface distinction;
- modified indicator remains visible independent of tab hover;
- tabs may show file-type icon only when it adds value.

Canonical comfortable/compact tab-height values are defined as **Design System component tokens** (current reference values 34 px / 30 px). Feature code must consume tokens rather than embed these numbers.

# Persona switch

Design / Photo is persistent and placed where it reads as a workspace mode, not a document tab. It must:

- preserve document selection where meaningful;
- change tool rail, context controls and preferred panels;
- never perform implicit rasterization/conversion;
- show shortcut and tooltip;
- animate minimally, mainly state/color and panel-content transition.

# Primary toolbar

Use at most three visual groups in normal widths. The toolbar contains high-frequency, context-independent or current-tool actions. Less important commands move to overflow.

Potential groups:

- leading: undo/redo, persona/workspace navigation, selection/tool context;
- center: current tool options/context controls;
- trailing: zoom/display controls, export/share-like action if applicable, overflow.

Every toolbar command must also exist in the menu/action system. Default toolbar grouping and overflow behavior are **V1 Required** and curated. User toolbar reordering/custom composition is a **Post-V1 Candidate** unless promoted by a milestone; workspace panel layout customization remains V1.

# Context toolbar

Context toolbar contents change based on active tool + selection. Examples:

- Move: X/Y/W/H, rotation, origin, snapping mode;
- Pen/Node: node type, convert, join, break, smooth/cusp, snapping;
- Shape: shape parameters, corner radius, fill/stroke summary;
- Text: font, weight, size, alignment, text style summary;
- Brush: size, hardness, opacity, flow, stabilizer;
- Selection: mode, feather, refine, mask behavior.

Rules:

- context changes must not cause wild vertical reflow;
- controls retain stable positions within tool families where possible;
- advanced options open a popover or Properties panel rather than overcrowding;
- overflow preserves complete access at narrow widths.

# Left tool rail

Default tool-rail width is a **Design System component token** (current reference range 44–48 px), not a literal owned by the shell feature. Tools are grouped with subtle separation.

Required behavior:

- icon-only by default;
- hover tooltip with name + primary shortcut + short description;
- click activates;
- long-press/right-click opens nested tool group;
- tiny corner/chevron marker may indicate grouped tools;
- double-click may open tool preferences only when a conventional behavior exists;
- active tool uses accent selection plus non-color cue;
- hidden/extra tools available through More and customization.

# Status bar

Status-bar height is a **Design System component token** (current reference range 26–30 px), not a hard-coded feature metric. It displays contextual, low-priority but valuable information without becoming a toolbar.

Left zone examples: zoom, current Surface/page, document dimensions/color mode.

Center: contextual hint, operation progress or selection summary.

Right: Snap, Guides, Grid, Proof Colors, pixel preview, background tasks indicator.

Status controls use compact hit areas but remain keyboard-addressable where actionable.

# Multi-window

Supported window types:

- main document window;
- detached document window;
- floating palette window;
- dedicated subsystem window when work benefits from persistent independent space;
- modal sheet/dialog owned by a window.

Remember geometry per display and clamp restored windows to visible desktop bounds. Mixed-DPI movement between monitors must update scaling without requiring restart.

# Full screen and distraction-free modes

- Full Screen: uses available display while preserving essential toolbar/panels.
- Focus Canvas: hides most chrome, preserves small escape affordance and keyboard control.
- Tab toggles: `Tab` may hide/show studio panels/toolbars by user-configurable shortcut.
- Never hide unsaved/progress/error conditions in a way that makes them undiscoverable.