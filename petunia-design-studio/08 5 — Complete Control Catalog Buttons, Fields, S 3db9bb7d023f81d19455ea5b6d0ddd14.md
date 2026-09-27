# 08.5 — Complete Control Catalog: Buttons, Fields, Sliders, Lists, Trees, Tables & Pickers

<aside>
🎚️

This is the canonical **control inventory**. New controls should be compositions/variants of these primitives before a new interaction primitive is invented.

</aside>

# Buttons

Variants:

- Primary: one dominant confirmation per modal/context.
- Secondary: ordinary action.
- Tertiary/ghost: low-chrome toolbar/panel action.
- Destructive: semantic red/error role; never use as default-focus primary unless confirmation context demands it.
- Icon-only: toolbar/panel compact action; mandatory tooltip/accessibility label.
- Split button: primary action + adjacent menu chevron when default action and variants coexist.

States: default, hover, pressed, focused, disabled, selected/toggled, busy. Busy buttons show spinner/progress and prevent duplicate activation where operation is not idempotent.

# Segmented controls

Use for 2–5 mutually exclusive modes visible simultaneously: alignment, preview modes, Persona-like micro modes. Do not use as generic tabs for large content sections.

# Toggle / checkbox / radio

- Toggle: immediate persistent boolean setting.
- Checkbox: selection/options often in forms/multi-choice.
- Radio: mutually exclusive choices when all options should be visible.

Indeterminate state required for mixed multi-selection.

# Text field

Anatomy: optional leading icon, editable value, optional trailing clear/unit/action.

States include validation error/warning, disabled, read-only, mixed, dirty/revert.

Behavior:

- select-all on explicit shortcut;
- numeric units never become ambiguous text;
- error message appears inline or tooltip with accessible description;
- commit on Enter/focus loss according to field semantics;
- Esc restores pre-edit value for transactional property editing when possible.

# Numeric field

Supports:

- keyboard entry;
- up/down stepper;
- scroll adjustment only when field focused or modifier intentionally held to avoid accidental changes;
- scrubbing by dragging label where expert workflow benefits;
- unit-aware numeric parsing (`2in`, `50%` where the property accepts that dimension) is **V1 Required**; general arithmetic expressions such as `10+5` are a **Post-V1 Candidate** behind a validated, side-effect-free parser;
- units shown explicitly;
- min/max clamping with feedback.

# Slider

Variants:

- continuous;
- stepped;
- range slider;
- slider + numeric field pair.

Slider anatomy includes track, fill, thumb, optional ticks and numeric readout. Fine adjustment with modifier key; double-click reset only when discoverable and reversible. Arrow keys adjust focused thumb.

# Scrubber

Compact expert control used for parameters like brush size or angle. Cursor changes; horizontal drag adjusts; Shift = fine, Alt/Option may adjust symmetrically if domain-specific. Always pair with visible value.

# Dropdown / popup button

Use for one selected value from a list. Shows current selection and chevron. Menus may include search for long lists (fonts, profiles, presets).

# Combobox

Editable/searchable dropdown for large enumerations. Keyboard first-result and exact text behavior must be predictable.

# Color control

Compact swatch button opens Color popover or focuses Color panel. Supports Fill/Stroke dual swatches, swap, reset/default, none/transparent indicator.

Color popover can include:

- model tabs or selector: RGB/CMYK/Lab/Gray;
- sliders/fields;
- alpha where applicable;
- hex only for RGB-relevant contexts;
- recent colors;
- document swatches;
- eyedropper action;
- profile/context hint.

# Gradient editor

Components:

- gradient preview strip;
- color stops;
- midpoint handles;
- opacity stops if model supports them;
- stop list/fields for precision;
- type selector Linear/Radial/etc.;
- angle/position controls;
- reverse;
- distribute stops;
- save as swatch.

# Stroke control

Summary row shows color, width, alignment. Expanded section contains cap, join, miter, dash pattern, start/end markers, pressure/width profile where available.

# Tabs

Use for peer views inside one bounded region: Layers/Channels/Paths, Properties/Color/Stroke. Tabs do not replace Persona/workspace semantics.

# Accordion/collapsible section

Panel section title + disclosure chevron + optional action. Persist open/closed state by workspace/user, not document. Avoid excessive nesting.

# Tree view

Required features for Layers/Assets trees:

- expand/collapse;
- disclosure triangle;
- icon/type glyph;
- thumbnail optional;
- label + inline rename;
- badges/status icons;
- visibility/lock columns where relevant;
- multi-selection;
- keyboard navigation;
- drag reorder/reparent;
- insertion line/target highlight;
- auto-scroll during drag;
- virtualization for scale.

# List and virtual list

Supports single/multi selection, keyboard movement, incremental search, context menu, reorder if applicable, row actions on hover. Long lists must not instantiate offscreen rows.

# Table/DataGrid

Used for Data Merge records, metadata, asset tables.

Features:

- sortable columns;
- resizable columns;
- optional reorder;
- fixed first columns where useful;
- multi-row selection;
- keyboard cell navigation;
- copy selected cells;
- virtualized rows;
- filter row/chips;
- empty/loading/error states;
- column visibility menu.

# Search field

See menu page; variants include local filter and global search. Never fake a search field as a generic text field without semantics/accessibility role.

# Stepper

For small discrete increments; often paired with numeric field. Press-and-hold repeat behavior must have acceleration limits.

# Progress

- spinner: indeterminate short task;
- linear progress: deterministic task;
- compact status progress: background tasks;
- detailed task row: import/export/batch operations with cancel/retry.

# Toast

Ephemeral nonblocking confirmation such as `Copied`, `Export complete`. Action button optional (`Show in Folder`). Errors requiring decision must not vanish as toast.

# Badge/chip/tag

Use sparingly for status/filter tokens. Removable chips have explicit close button and keyboard semantics.

# Breadcrumb

Used where hierarchical resource navigation helps, not as decoration. Each ancestor interactive; overflow collapses middle ancestors first.

# Menu button / More

Ellipsis means additional related actions, never a dumping ground for discoverability failures.

# Scrollbar

Overlay/subtle by default where platform style fits, but always visible enough during active scrolling/hover. Large-canvas scrollbars optional depending navigation model. Track click/drag behavior follows platform conventions.

# Split view

Resizable panels with minimum size and collapse behavior. Splitter has semantic role and keyboard resizing where feasible.

# Inspector/property row conventions

Labels align consistently per section. Units, reset/revert, **binding/automation indicators only for capabilities that actually exist**, and mixed states have reserved positions to prevent layout jitter. **Keyframe/animation semantics are Out of Scope unless a future product ADR introduces animation; do not reserve or implement animation UI merely from this layout note.**