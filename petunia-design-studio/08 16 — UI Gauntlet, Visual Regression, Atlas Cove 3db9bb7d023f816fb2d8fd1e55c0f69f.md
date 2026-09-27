# 08.16 — UI Gauntlet, Visual Regression, Atlas Coverage & No-Gap Checklist

# Purpose

The Atlas is only useful if implementation can be checked against it. This page defines UI quality gates and a coverage checklist intended to prevent forgotten states and subsystem windows.

# Component-state gauntlet

Every reusable component must be reviewed/tested in:

- default;
- hover;
- pressed;
- keyboard focus;
- disabled;
- selected/toggled;
- mixed/indeterminate when applicable;
- loading/busy;
- error;
- warning;
- long-label/localized;
- dark/light;
- compact/comfortable/spacious;
- 100%/150%/200% DPI-equivalent scale.

# Shell gauntlet

Test:

- minimum supported window;
- ultrawide;
- two monitors;
- mixed DPI;
- maximize/restore;
- full screen;
- detach/reattach document;
- crash/recovery restart;
- workspace restore with missing monitor.

# Docking gauntlet

- nested splits;
- tab reorder;
- float/redock;
- collapse/expand;
- min-size pressure;
- serialize/restore;
- Persona workspace switch;
- keyboard panel movement;
- 20+ panel stress layout.

# Layers/tree gauntlet

- 10k rows normal benchmark;
- 100k synthetic stress where supported;
- deep hierarchy;
- rapid document deltas;
- multi-selection;
- rename;
- drag/reparent;
- visibility/lock toggles;
- thumbnail updates;
- keyboard navigation;
- screen-reader semantics.

# Canvas gauntlet

- extreme zoom in/out;
- large coordinate values;
- high-density nodes;
- overlays combined;
- guides/grid/snap;
- continuous pointer/pen input;
- selection over white/black/high-detail art;
- HiDPI handle sizing;
- smooth viewport while panels update.

# Dialog/window gauntlet

For every dialog: keyboard-only completion, Esc behavior, default action, validation, min size, long localization, screen-reader labels, reopening preserved state where intended.

Must explicitly cover:

- New Document;
- Import Interpretation;
- Missing Fonts/Links;
- Export;
- Batch Export;
- Data Source Import;
- Data Merge Generate;
- Preferences;
- Shortcut Capture;
- Recovery;
- destructive confirmation;
- About/License/Third-party notices.

# Visual regression

Golden screenshots should cover stable component gallery and canonical shell scenarios, not rely solely on full-app screenshots. Use semantic/state assertions alongside pixels to avoid fragile false confidence.

Canonical golden scenes:

- Design default;
- Photo default;
- compact density;
- light theme;
- floating palettes;
- modal Export;
- command palette;
- preferences;
- Data Merge preview;
- error/banner/toast stack.

# Performance budgets

Track:

- frame time/P95 during normal UI interaction;
- Layers scrolling;
- docking drag;
- typing latency;
- command palette search;
- panel resize while canvas animates;
- histogram/background-task interference;
- memory use of large trees/assets.

Petunia diagnostics, Slint instrumentation and renderer/profiler telemetry feed developer builds; UI performance evidence must record the actual tooling and environment used.

# Accessibility gate

Release candidate cannot ship with inaccessible primary workflow. Required keyboard/screen-reader paths:

New/Open → select object → inspect Properties → change value → export.

Photo: select layer → adjustment → edit curve/value → save/export.

# Localization gate

Run pseudo-localization plus real samples: pt-BR, German expansion, Japanese/CJK IME, Arabic/RTL. No clipped primary actions or unreadable menus.

# Error/recovery gate

Inject:

- disk full/read-only;
- missing linked asset;
- unavailable profile/font;
- corrupt import;
- export write failure;
- plugin failure;
- background task cancellation.

UI must explain and recover without document corruption.

# Atlas coverage checklist

| Area | Canonical page | Must exist before UI freeze |
| --- | --- | --- |
| Tokens/visual system | 08.1 | Yes |
| Shell/toolbars/tabs/status | 08.2 | Yes |
| Docking/panels/floating | 08.3 | Yes |
| Menus/actions/shortcuts/search | 08.4 | Yes |
| Control catalog | 08.5 | Yes |
| Canvas/navigation/snapping | 08.6 | Yes |
| Design Persona | 08.7 | Yes |
| Photo Persona | 08.8 | Yes |
| File/import/relink/recovery | 08.9 | Yes |
| Export/batch output | 08.10 | Yes |
| Data Merge | 08.11 | Yes |
| Preferences/customization/help | 08.12 | Yes |
| Feedback/modality/errors | 08.13 | Yes |
| Accessibility/input/localization | 08.14 | Yes |
| Slint implementation map | 08.15 | Yes |
| QA/regression/gates | 08.16 | Yes |
| Canonical panel inventory | 08.17 | Yes |
| Window/dialog/subsystem inventory | 08.18 | Yes |
| Microcopy/terminology/units | 08.19 | Yes |
| UX/usability/cognitive load | 08.20 | Yes |
| Design-system governance/portability | 08.21 | Yes |
| Affinity reference/divergence rules | 08.22 | Yes |
| Tool interaction grammar | 08.23 | Yes |
| Design tool-by-tool UX | 08.24 | Yes |
| Design panel/detail UX | 08.25 | Yes |
| Typography/text editing UX | 08.26 | Yes |
| Precision/snapping feedback | 08.27 | Yes |
| Cross-Persona/Workspace Profiles | 08.28 | Yes |
| Import/export/preflight completion flow | 08.29 | Yes |
| UI/UX evidence corpus | 08.30 | Yes |
| Photo tool-by-tool UX | 08.31 | Yes |
| Photo adjustment/mask/analysis UX | 08.32 | Yes |
| Affinity tool/panel coverage ledger | 08.33 | Yes |

# Scope-status gate

Every UI feature named by the Atlas must resolve to **one of the canonical requirement states in 12.8, with no UI-specific parallel taxonomy**: `V1 Required`, `Milestone Required`, `Post-V1 Candidate`, `Research / Prior Art`, `Open ADR`, `Deferred by Dependency`, `Deprecated / Migration Only`, `Historical`, or `Out of Scope`. Unqualified words such as `later`, `future`, `planned`, `if supported`, `when implemented` or `optional` must not be used as roadmap authority. The Functional Atlas/Product Charter owns feature scope; the Interface Atlas owns presentation and interaction once scope is known. Repository/frontmatter machine values may use normalized slugs such as `v1-required`, but they map one-to-one to 12.8. CI/documentation review must flag user-visible capabilities whose UI page and functional status disagree.

# Change rule

If implementation introduces a persistent UI concept not represented by this Atlas, the feature is incomplete until the Atlas is amended with anatomy, states, keyboard/accessibility behavior, error/loading state and workspace persistence rules where relevant.

# Usability and Design System gates

UI quality now includes two additional canonical dimensions:

- **08.20 UX/usability:** representative end-to-end jobs must be discoverable, recoverable, efficient after learning and testable without relying on screenshots alone. Track first-attempt success, wrong-mode errors, command discovery, recovery and repeated-task efficiency as signals.
- **08.21 Design System governance:** feature UI must resolve copy/icons/tokens/help/a11y metadata semantically, pass no-hardcode checks and avoid GPUI types in domain/application contracts.

# Portability gauntlet

At milestone boundaries run a toolkit-free/mock-shell scenario through `PetuniaGuiBridge`: enumerate actions, query presentation models/properties, submit edits, observe jobs/diagnostics and exercise representative document workflows without constructing GPUI widgets. Failure means UI coupling leaked into core.

# Tokenization gauntlet

CI/review should flag direct literal colors, icon/SVG paths, user-visible UI strings, ad-hoc spacing/radius/motion values or direct help URLs in feature modules outside approved Design System/resource definitions.

# Agent-driven UI review

Code agents must validate semantics in addition to pixels: focus order, accessible roles/names, action IDs, TextId/IconId/token resolution, disabled reason, empty/loading/error states and the expected Action/Command result. A visually matching screenshot cannot override semantic failures.