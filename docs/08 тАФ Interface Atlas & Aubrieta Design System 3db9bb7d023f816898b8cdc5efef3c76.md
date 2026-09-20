# 08 — Interface Atlas & Aubrieta Design System

<aside>
🗺️

**Canonical UI specification.** This atlas defines the intended Aubrieta Design Suite desktop interface at implementation level. GPUI is the primary shell; the specification is expressed semantically so behavior remains portable through `AubrietaGuiBridge`.

</aside>

# Product UI direction

Aubrieta follows an **Affinity-Studio-like creative workflow** with a more modern, calm and clean desktop language. The visual system takes inspiration from contemporary Apple platform principles without cloning Apple chrome: content-first hierarchy, restrained accent color, familiar control behavior, clear modality, predictable menus, strong keyboard support, refined density and subtle depth.

The interface must feel simultaneously:

- **professional** enough for long creative sessions;
- **dense** enough for expert workflows;
- **calm** enough to avoid visual fatigue;
- **discoverable** enough for new users;
- **fast** enough to feel native at 120 Hz where hardware permits;
- **customizable** without becoming visually inconsistent;
- **accessible** by keyboard, assistive technology and non-color state cues.

# Non-negotiable UI principles

1. **Content owns attention.** Chrome is quieter than canvas/document content.
2. **Reuse behavior, own presentation.** Prefer GPUI Kit/`gpui-base` interaction mechanics; Aubrieta owns visual tokens and component styling.
3. **No hidden-only essential actions.** Context menus and tooltips accelerate workflows but never become the only access path to an essential command.
4. **Selection drives context.** Context toolbar, Properties and status information respond to active Persona, tool and selection.
5. **One semantic action system.** Menu bar, toolbar, context menu, shortcut, command palette, plugin and MCP resolve to the same Aubrieta Action.
6. **Progressive disclosure.** Common controls remain visible; advanced settings expand, pop over or move into dedicated subsystem dialogs.
7. **Modality is expensive.** Prefer panels/popovers for reversible inspection; use modal sheets/dialogs only for tasks that require focused confirmation or multi-step completion.
8. **Desktop-first precision.** Mouse, trackpad, pen, keyboard and high-DPI screens are first-class.
9. **Document state and workspace state are different.** Docking, panel visibility, window geometry and Persona arrangement are never canonical document content.
10. **Every presentation resource is tokenized/semantic.** Spacing, radius, typography, motion, shadows, separators, hit areas and state colors are design tokens; user-visible copy resolves through `TextId`; icons through `IconId`; cursors/assets through semantic resource IDs. Feature UI must not use literal English strings, literal theme colors or direct icon file paths as canonical identity.

# External design references

- [Apple Human Interface Guidelines — Toolbars](https://developer.apple.com/design/human-interface-guidelines/toolbars)
- [Apple Human Interface Guidelines — Modality](https://developer.apple.com/design/human-interface-guidelines/modality)
- [Apple Human Interface Guidelines — Context menus](https://developer.apple.com/design/human-interface-guidelines/context-menus)
- [Apple Design Resources](https://developer.apple.com/design/resources/)
- [GPUI Kit](https://gpui-kit.com/)

# Atlas contract

Each child page is both **design specification** and **implementation acceptance criteria**. When implementation diverges, update an ADR or this atlas; do not allow undocumented UI behavior to become canonical by accident.

# Atlas index

This table is the **canonical navigation map** for the interface specification. A persistent UI feature is considered undocumented if it cannot be mapped to one of these rows.

| ID | Area | Canonical page | Primary implementation concern |
| --- | --- | --- | --- |
| 08.1 | Visual foundations | [08.1 — Visual Foundations, Tokens, Typography, Spacing & Motion](08%201%20%E2%80%94%20Visual%20Foundations,%20Tokens,%20Typography,%20Spa%203db9bb7d023f81cb9d14fd9f7e67d2c7.md) | tokens, typography, spacing, radii, color, elevation, motion, density |
| 08.2 | Application shell | [08.2 — Application Shell, Window Anatomy, Tabs, Toolbars & Status Bar](08%202%20%E2%80%94%20Application%20Shell,%20Window%20Anatomy,%20Tabs,%20To%203db9bb7d023f81069063d901399c6bdf.md) | windows, tabs, Persona switch, toolbars, tool rail, status bar |
| 08.3 | Docking and panels | [08.3 — Docking, Panels, Inspectors, Floating Palettes & Workspace Presets](08%203%20%E2%80%94%20Docking,%20Panels,%20Inspectors,%20Floating%20Palet%203db9bb7d023f81259b36ea98ef7db4ca.md) | dock tree, splits, floating palettes, panel anatomy, workspace presets |
| 08.4 | Menus and commands | [08.4 — Menus, Submenus, Context Menus, Command Palette, Shortcuts & Search](08%204%20%E2%80%94%20Menus,%20Submenus,%20Context%20Menus,%20Command%20Pal%203db9bb7d023f817f9cc5ebb4ee801d29.md) | menu bar, submenus, context menus, command palette, shortcuts, search |
| 08.5 | Control catalog | [08.5 — Complete Control Catalog: Buttons, Fields, Sliders, Lists, Trees, Tables & Pickers](08%205%20%E2%80%94%20Complete%20Control%20Catalog%20Buttons,%20Fields,%20S%203db9bb7d023f81d19455ea5b6d0ddd14.md) | buttons, fields, sliders, pickers, tabs, trees, lists, tables, progress |
| 08.6 | Canvas interaction | [08.6 — Canvas, Rulers, Guides, Zoom, Selection, Snapping & On-Canvas HUD](08%206%20%E2%80%94%20Canvas,%20Rulers,%20Guides,%20Zoom,%20Selection,%20Sn%203db9bb7d023f816f910ad0aadc8984ea.md) | rulers, guides, grid, zoom, selection, snapping, HUD, overlays, cursors |
| 08.7 | Design Persona | [08.7 — Design Persona: Complete Interface Specification](08%207%20%E2%80%94%20Design%20Persona%20Complete%20Interface%20Specifica%203db9bb7d023f817dae7dc5b0ad914897.md) | vector tools, Layers, Properties, color/stroke, typography, Surfaces, assets |
| 08.8 | Photo Persona | [08.8 — Photo Persona: Complete Interface Specification](08%208%20%E2%80%94%20Photo%20Persona%20Complete%20Interface%20Specificat%203db9bb7d023f813c830fcd0c1e439524.md) | brushes, selections, masks, adjustments, channels, histogram, crop |
| 08.9 | File workflows | [08.9 — Welcome, New/Open, Place/Import, Relink, Missing Assets & File Workflows](08%209%20%E2%80%94%20Welcome,%20New%20Open,%20Place%20Import,%20Relink,%20Mi%203db9bb7d023f81878276df040ba2c753.md) | welcome, new/open, place/import, relink, missing assets/fonts, recovery |
| 08.10 | Export/output | [08.10 — Export, Quick Export, Batch Export & Output Window](08%2010%20%E2%80%94%20Export,%20Quick%20Export,%20Batch%20Export%20&%20Outpu%203db9bb7d023f81b5a056f4d09e9b21fb.md) | quick export, full Export window, format options, batch, presets, progress |
| 08.11 | Variable Data | [08.11 — Variable Data / Data Merge Window, Bindings, Preview & Generation](08%2011%20%E2%80%94%20Variable%20Data%20Data%20Merge%20Window,%20Bindings,%203db9bb7d023f8154b090cbb0f5d63b3d.md) | data sources, fields, bindings, record preview, table, generate/export |
| 08.12 | Preferences and customization | [08.12 — Preferences, Customization, Themes, Icons, Workspaces, Onboarding & Help](08%2012%20%E2%80%94%20Preferences,%20Customization,%20Themes,%20Icons,%203db9bb7d023f8141b9d8e64e35e00faa.md) | theme, density, icons, workspace, shortcuts, canvas, color, plugins, help |
| 08.13 | Feedback and modality | [08.13 — Tooltips, Popovers, Dialogs, Alerts, Notifications, Errors & Progress](08%2013%20%E2%80%94%20Tooltips,%20Popovers,%20Dialogs,%20Alerts,%20Notif%203db9bb7d023f816fa292e7fc620a84ba.md) | tooltips, popovers, dialogs, alerts, toasts, banners, errors, progress |
| 08.14 | Accessibility and input | [08.14 — Accessibility, Keyboard, Pointer, Pen, IME, Localization, HiDPI & Adaptive Density](08%2014%20%E2%80%94%20Accessibility,%20Keyboard,%20Pointer,%20Pen,%20IME%203db9bb7d023f8167a9bdff182768f9f0.md) | keyboard, focus, screen readers, pointer, pen, IME, localization, HiDPI |
| 08.15 | GPUI implementation map | [08.15 — GPUI / GPUI Kit Implementation Map & Component Ownership](08%2015%20%E2%80%94%20GPUI%20GPUI%20Kit%20Implementation%20Map%20&%20Compone%203db9bb7d023f81bc8399f58f22c4e67b.md) | GPUI/Kit/base mapping, Aubrieta ownership, crates/modules, design-system wrappers |
| 08.16 | UI quality gates | [08.16 — UI Gauntlet, Visual Regression, Atlas Coverage & No-Gap Checklist](08%2016%20%E2%80%94%20UI%20Gauntlet,%20Visual%20Regression,%20Atlas%20Cove%203db9bb7d023f816fb2d8fd1e55c0f69f.md) | gauntlets, regression, performance, accessibility, localization, no-gap checklist |
| 08.17 | Panel inventory | [08.17 — Panel Atlas: Every Dockable Inspector, Browser & Utility Panel](08%2017%20%E2%80%94%20Panel%20Atlas%20Every%20Dockable%20Inspector,%20Brow%203db9bb7d023f81dfa0a2c4642a3c110e.md) | all dockable panels, ownership, empty/loading/error states |
| 08.18 | Window/dialog inventory | [08.18 — Window & Dialog Atlas: Every Modal, Sheet, Manager & Subsystem Window](08%2018%20%E2%80%94%20Window%20&%20Dialog%20Atlas%20Every%20Modal,%20Sheet,%20%203db9bb7d023f81dab281e2ca22b799ad.md) | all dialogs, managers, subsystem windows, Print, profiles, About/notices |
| 08.19 | Interface language | [08.19 — Microcopy, Labels, Naming, States, Units & Interface Language](08%2019%20%E2%80%94%20Microcopy,%20Labels,%20Naming,%20States,%20Units%20&%203db9bb7d023f817a8786c1aa38209a4a.md) | voice, labels, errors, units, action naming and terminology |
| 08.20 | UX and usability | [08.20 — UX, Usability, Information Architecture, Cognitive Load & User Research](08%2020%20%E2%80%94%20UX,%20Usability,%20Information%20Architecture,%20C%203db9bb7d023f81c38d98c1ddc9a6956d.md) | learnability, discoverability, cognitive load, jobs-to-be-done, usability metrics and recovery |
| 08.21 | Design System governance and portability | [08.21 — Design System Governance, Token Enforcement, Customization & UI Portability Contract](08%2021%20%E2%80%94%20Design%20System%20Governance,%20Token%20Enforcemen%203db9bb7d023f819e90cbc2da911b8c03.md) | no-hardcode rules, semantic resources, component contracts, token enforcement and toolkit portability |
| 08.22 | Affinity reference and divergence | [08.22 — Affinity Reference Model, Aubrieta Divergence Rules & Workflow Vocabulary](08%2022%20%E2%80%94%20Affinity%20Reference%20Model,%20Aubrieta%20Diverge%203df9bb7d023f815cbd73efe081adc2d6.md) | Affinity prior art, Adopt/Adapt/Reject discipline, Workspace Profiles, deliberate Aubrieta improvements |
| 08.23 | Tool interaction grammar | [08.23 — Tool Interaction Grammar, Context Toolbar, Modifiers & Canvas HUD Contract](08%2023%20%E2%80%94%20Tool%20Interaction%20Grammar,%20Context%20Toolbar,%203df9bb7d023f8144bba3db77c199a947.md) | tool state machine, context-toolbar order, modifiers, handles, preview/commit/cancel, numeric takeover |
| 08.24 | Design tool UX | [08.24 — Design Persona Tool-by-Tool UX Contract & Affinity Mapping](08%2024%20%E2%80%94%20Design%20Persona%20Tool-by-Tool%20UX%20Contract%20&%20%203df9bb7d023f81e7a45bdac3b72a71ea.md) | tool-by-tool Affinity mapping, Aubrieta improvements, fixtures and interaction contracts |
| 08.25 | Design panels and inspectors | [08.25 — Panels, Layers, Appearance, Colour, Stroke, Transform, Assets & Symbols UX Contract](08%2025%20%E2%80%94%20Panels,%20Layers,%20Appearance,%20Colour,%20Stroke%203df9bb7d023f814686f4e75513c11bd0.md) | structural Layers, contextual Properties, Appearance, Colour, Stroke, Assets, Symbols and panel states |
| 08.26 | Typography and text UX | [08.26 — Typography, Text Editing, Layout & Precision UX Contract](08%2026%20%E2%80%94%20Typography,%20Text%20Editing,%20Layout%20&%20Precisi%203df9bb7d023f81e5a00acd3217f200f6.md) | on-canvas editing, text frames/path text, font/style panels, complex scripts, missing fonts |
| 08.27 | Precision and snapping UX | [08.27 — Snapping, Guides, Measurement, Numeric Precision & Spatial Feedback](08%2027%20%E2%80%94%20Snapping,%20Guides,%20Measurement,%20Numeric%20Pre%203df9bb7d023f81409b2df9edb911f2b2.md) | global/local snapping, provenance, guides/grids, measurements, units, coordinate spaces |
| 08.28 | Personas and mixed workspaces | [08.28 — Design ↔ Photo Personas, Mixed Workspace Profiles & Cross-Discipline Flow](08%2028%20%E2%80%94%20Design%20%E2%86%94%20Photo%20Personas,%20Mixed%20Workspace%20P%203df9bb7d023f81faaae5e02b4a6eb1b3.md) | canonical Personas, mixed Workspace Profiles, customization/share/reset and cross-discipline flow |
| 08.29 | Workflow completion | [08.29 — Import, Place, Export, Preflight & Workflow Completion UX](08%2029%20%E2%80%94%20Import,%20Place,%20Export,%20Preflight%20&%20Workflo%203df9bb7d023f81c8b74ff301ce199160.md) | place/import, preflight, quick/full/batch export, production warnings and recovery |
| 08.30 | UI/UX evidence corpus | [08.30 — UI/UX Evidence Corpus, Interaction Conformance & Human-Factors Gauntlet](08%2030%20%E2%80%94%20UI%20UX%20Evidence%20Corpus,%20Interaction%20Conform%203df9bb7d023f81098725d56753eb0372.md) | semantic goldens, workflow fixtures, interaction traces, usability studies and release evidence |
| 08.31 | Photo tool UX | [08.31 — Photo Persona Tool-by-Tool UX Contract & Affinity Pixel/Photo Mapping](08%2031%20%E2%80%94%20Photo%20Persona%20Tool-by-Tool%20UX%20Contract%20&%20A%203df9bb7d023f810cada6d30e5b867888.md) | selection, crop, paint, erase, clone/heal/inpaint, retouch, pen input and destructive-target safety |
| 08.32 | Photo panels and nondestructive editing | [08.32 — Photo Layers, Adjustments, Masks, Live Filters, Channels & Analysis UX](08%2032%20%E2%80%94%20Photo%20Layers,%20Adjustments,%20Masks,%20Live%20Fil%203df9bb7d023f81609273c08be65cd5a2.md) | adjustment layers, live filters, masks, channels, histogram, Brushes and analysis panels |
| 08.33 | Affinity coverage ledger | [08.33 — Affinity Tool & Panel Coverage Ledger → Aubrieta Scope/Authority Matrix](08%2033%20%E2%80%94%20Affinity%20Tool%20&%20Panel%20Coverage%20Ledger%20%E2%86%92%20Au%203df9bb7d023f812684f9d8b0bdb12c46.md) | official Affinity tool/panel inventory mapped to Adopt/Adapt/Reference and canonical Aubrieta authority |

# Coverage rule

Before a persistent UI feature is accepted, its specification must answer **all** applicable questions: anatomy, dimensions/tokens, default state, hover/pressed/focus/disabled/mixed/loading/error states, keyboard behavior, pointer/drag behavior, accessibility semantics, localization expansion, persistence scope, undo/transaction behavior, error/recovery path, performance/virtualization needs, and mapping to Aubrieta Action/Command when actionable.

# Reference concept

The intended family is **Affinity-like creative workflow + GPUI-native information density + Apple-like restraint and familiarity**, with Aubrieta owning the final identity. Accent color is used intentionally for primary actions/status/selection instead of flooding persistent chrome; toolbars prioritize common actions and collapse secondary items to overflow; modality is reserved for focused tasks rather than routine inspection.

[08.1 — Visual Foundations, Tokens, Typography, Spacing & Motion](08%201%20%E2%80%94%20Visual%20Foundations,%20Tokens,%20Typography,%20Spa%203db9bb7d023f81cb9d14fd9f7e67d2c7.md)

[08.2 — Application Shell, Window Anatomy, Tabs, Toolbars & Status Bar](08%202%20%E2%80%94%20Application%20Shell,%20Window%20Anatomy,%20Tabs,%20To%203db9bb7d023f81069063d901399c6bdf.md)

[08.3 — Docking, Panels, Inspectors, Floating Palettes & Workspace Presets](08%203%20%E2%80%94%20Docking,%20Panels,%20Inspectors,%20Floating%20Palet%203db9bb7d023f81259b36ea98ef7db4ca.md)

[08.4 — Menus, Submenus, Context Menus, Command Palette, Shortcuts & Search](08%204%20%E2%80%94%20Menus,%20Submenus,%20Context%20Menus,%20Command%20Pal%203db9bb7d023f817f9cc5ebb4ee801d29.md)

[08.5 — Complete Control Catalog: Buttons, Fields, Sliders, Lists, Trees, Tables & Pickers](08%205%20%E2%80%94%20Complete%20Control%20Catalog%20Buttons,%20Fields,%20S%203db9bb7d023f81d19455ea5b6d0ddd14.md)

[08.6 — Canvas, Rulers, Guides, Zoom, Selection, Snapping & On-Canvas HUD](08%206%20%E2%80%94%20Canvas,%20Rulers,%20Guides,%20Zoom,%20Selection,%20Sn%203db9bb7d023f816f910ad0aadc8984ea.md)

[08.7 — Design Persona: Complete Interface Specification](08%207%20%E2%80%94%20Design%20Persona%20Complete%20Interface%20Specifica%203db9bb7d023f817dae7dc5b0ad914897.md)

[08.8 — Photo Persona: Complete Interface Specification](08%208%20%E2%80%94%20Photo%20Persona%20Complete%20Interface%20Specificat%203db9bb7d023f813c830fcd0c1e439524.md)

[08.9 — Welcome, New/Open, Place/Import, Relink, Missing Assets & File Workflows](08%209%20%E2%80%94%20Welcome,%20New%20Open,%20Place%20Import,%20Relink,%20Mi%203db9bb7d023f81878276df040ba2c753.md)

[08.10 — Export, Quick Export, Batch Export & Output Window](08%2010%20%E2%80%94%20Export,%20Quick%20Export,%20Batch%20Export%20&%20Outpu%203db9bb7d023f81b5a056f4d09e9b21fb.md)

[08.11 — Variable Data / Data Merge Window, Bindings, Preview & Generation](08%2011%20%E2%80%94%20Variable%20Data%20Data%20Merge%20Window,%20Bindings,%203db9bb7d023f8154b090cbb0f5d63b3d.md)

[08.12 — Preferences, Customization, Themes, Icons, Workspaces, Onboarding & Help](08%2012%20%E2%80%94%20Preferences,%20Customization,%20Themes,%20Icons,%203db9bb7d023f8141b9d8e64e35e00faa.md)

[08.13 — Tooltips, Popovers, Dialogs, Alerts, Notifications, Errors & Progress](08%2013%20%E2%80%94%20Tooltips,%20Popovers,%20Dialogs,%20Alerts,%20Notif%203db9bb7d023f816fa292e7fc620a84ba.md)

[08.14 — Accessibility, Keyboard, Pointer, Pen, IME, Localization, HiDPI & Adaptive Density](08%2014%20%E2%80%94%20Accessibility,%20Keyboard,%20Pointer,%20Pen,%20IME%203db9bb7d023f8167a9bdff182768f9f0.md)

[08.15 — GPUI / GPUI Kit Implementation Map & Component Ownership](08%2015%20%E2%80%94%20GPUI%20GPUI%20Kit%20Implementation%20Map%20&%20Compone%203db9bb7d023f81bc8399f58f22c4e67b.md)

[08.16 — UI Gauntlet, Visual Regression, Atlas Coverage & No-Gap Checklist](08%2016%20%E2%80%94%20UI%20Gauntlet,%20Visual%20Regression,%20Atlas%20Cove%203db9bb7d023f816fb2d8fd1e55c0f69f.md)

[08.17 — Panel Atlas: Every Dockable Inspector, Browser & Utility Panel](08%2017%20%E2%80%94%20Panel%20Atlas%20Every%20Dockable%20Inspector,%20Brow%203db9bb7d023f81dfa0a2c4642a3c110e.md)

[08.18 — Window & Dialog Atlas: Every Modal, Sheet, Manager & Subsystem Window](08%2018%20%E2%80%94%20Window%20&%20Dialog%20Atlas%20Every%20Modal,%20Sheet,%20%203db9bb7d023f81dab281e2ca22b799ad.md)

[08.19 — Microcopy, Labels, Naming, States, Units & Interface Language](08%2019%20%E2%80%94%20Microcopy,%20Labels,%20Naming,%20States,%20Units%20&%203db9bb7d023f817a8786c1aa38209a4a.md)

[08.20 — UX, Usability, Information Architecture, Cognitive Load & User Research](08%2020%20%E2%80%94%20UX,%20Usability,%20Information%20Architecture,%20C%203db9bb7d023f81c38d98c1ddc9a6956d.md)

[08.21 — Design System Governance, Token Enforcement, Customization & UI Portability Contract](08%2021%20%E2%80%94%20Design%20System%20Governance,%20Token%20Enforcemen%203db9bb7d023f819e90cbc2da911b8c03.md)

[08.22 — Affinity Reference Model, Aubrieta Divergence Rules & Workflow Vocabulary](08%2022%20%E2%80%94%20Affinity%20Reference%20Model,%20Aubrieta%20Diverge%203df9bb7d023f815cbd73efe081adc2d6.md)

[08.23 — Tool Interaction Grammar, Context Toolbar, Modifiers & Canvas HUD Contract](08%2023%20%E2%80%94%20Tool%20Interaction%20Grammar,%20Context%20Toolbar,%203df9bb7d023f8144bba3db77c199a947.md)

[08.24 — Design Persona Tool-by-Tool UX Contract & Affinity Mapping](08%2024%20%E2%80%94%20Design%20Persona%20Tool-by-Tool%20UX%20Contract%20&%20%203df9bb7d023f81e7a45bdac3b72a71ea.md)

[08.25 — Panels, Layers, Appearance, Colour, Stroke, Transform, Assets & Symbols UX Contract](08%2025%20%E2%80%94%20Panels,%20Layers,%20Appearance,%20Colour,%20Stroke%203df9bb7d023f814686f4e75513c11bd0.md)

[08.26 — Typography, Text Editing, Layout & Precision UX Contract](08%2026%20%E2%80%94%20Typography,%20Text%20Editing,%20Layout%20&%20Precisi%203df9bb7d023f81e5a00acd3217f200f6.md)

[08.27 — Snapping, Guides, Measurement, Numeric Precision & Spatial Feedback](08%2027%20%E2%80%94%20Snapping,%20Guides,%20Measurement,%20Numeric%20Pre%203df9bb7d023f81409b2df9edb911f2b2.md)

[08.28 — Design ↔ Photo Personas, Mixed Workspace Profiles & Cross-Discipline Flow](08%2028%20%E2%80%94%20Design%20%E2%86%94%20Photo%20Personas,%20Mixed%20Workspace%20P%203df9bb7d023f81faaae5e02b4a6eb1b3.md)

[08.29 — Import, Place, Export, Preflight & Workflow Completion UX](08%2029%20%E2%80%94%20Import,%20Place,%20Export,%20Preflight%20&%20Workflo%203df9bb7d023f81c8b74ff301ce199160.md)

[08.30 — UI/UX Evidence Corpus, Interaction Conformance & Human-Factors Gauntlet](08%2030%20%E2%80%94%20UI%20UX%20Evidence%20Corpus,%20Interaction%20Conform%203df9bb7d023f81098725d56753eb0372.md)

[08.31 — Photo Persona Tool-by-Tool UX Contract & Affinity Pixel/Photo Mapping](08%2031%20%E2%80%94%20Photo%20Persona%20Tool-by-Tool%20UX%20Contract%20&%20A%203df9bb7d023f810cada6d30e5b867888.md)

[08.32 — Photo Layers, Adjustments, Masks, Live Filters, Channels & Analysis UX](08%2032%20%E2%80%94%20Photo%20Layers,%20Adjustments,%20Masks,%20Live%20Fil%203df9bb7d023f81609273c08be65cd5a2.md)

[08.33 — Affinity Tool & Panel Coverage Ledger → Aubrieta Scope/Authority Matrix](08%2033%20%E2%80%94%20Affinity%20Tool%20&%20Panel%20Coverage%20Ledger%20%E2%86%92%20Au%203df9bb7d023f812684f9d8b0bdb12c46.md)