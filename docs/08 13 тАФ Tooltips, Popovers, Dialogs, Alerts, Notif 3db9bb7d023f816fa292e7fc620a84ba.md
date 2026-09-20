# 08.13 — Tooltips, Popovers, Dialogs, Alerts, Notifications, Errors & Progress

# Interaction surface hierarchy

Use the lightest-weight surface that solves the task:

1. inline hint/state;
2. tooltip;
3. popover;
4. nonmodal panel/palette;
5. sheet/dialog;
6. dedicated subsystem window;
7. alert/critical confirmation.

# Tooltips

Tooltips are mandatory for icon-only controls and unfamiliar creative-tool icons.

Default content:

- tool/action name;
- shortcut if assigned;
- one short explanatory sentence when meaning is not obvious;
- optional `?`/Learn More for complex operations.

Timing:

- initial hover delay ~500–700 ms;
- faster subsequent tooltip delay within same interaction session;
- immediate on keyboard focus after short accessibility-friendly delay if appropriate.

Tooltips avoid covering pointer target and critical canvas content. They dismiss on click, pointer leave, Esc or significant context change.

# Rich tooltips

Use sparingly for complex tools. May include icon, title, 1–2 sentence description, shortcut and help link. No tutorials inside tooltips.

# Popovers

Use for temporary parameter choice/color/presets. Popover anchors to invoking control, flips/clamps to window bounds, and closes on outside click/Esc unless interaction requires persistence.

Popover may be pinnable into panel only if a clear reusable workflow justifies it.

# Menus vs popovers

Menu = command choice.

Popover = controls/content/parameters.

Do not place sliders/text fields inside ordinary menus when a popover is semantically clearer.

# Dialogs

Dialog anatomy:

- title;
- optional concise description;
- content sections;
- validation/error region;
- action row.

Default action on trailing/right according to platform conventions while preserving cross-platform consistency. Escape invokes Cancel when safe. Enter invokes default action only when current field does not need multiline/alternate handling.

# Sheets

Use owned sheet-like modality for document-scoped tasks: Export, New Document, Import Interpretation. Sheet blocks editing in owner context but not unrelated document windows when architecture allows.

# Destructive confirmation

Confirmation text names object/consequence: `Delete 4 selected layers?` rather than `Are you sure?`. Buttons use `Delete` / `Cancel`. Include irreversible consequence only if no undo/recovery exists.

Do not confirm every reversible delete; rely on Undo when safe.

# Alerts

Alerts reserved for conditions requiring immediate user decision or blocking continuation. Never stack multiple alerts. Detailed technical information goes behind `Details` disclosure/copy button.

# Inline validation

Preferred for form errors. Field border/state + message + accessibility description. Validation does not erase entered value.

# Banners

Nonmodal window/document-level issue: missing links, color-profile warning, autosave failure. Banner can contain one primary action + dismiss/more. Persist important unresolved issues across relevant session context.

# Toasts

Use for transient success/info:

- Copied;
- Preset saved;
- Export complete.

Optional action: Undo/Show. Maximum simultaneous visible toasts bounded; queue/coalesce repeated events.

# Notifications center / task popover

Background tasks icon in status region opens list:

- thumbnail/icon;
- task name;
- progress;
- status;
- cancel/retry/reveal action;
- errors.

Completed tasks auto-expire from list after sensible time but remain in log if needed.

# Progress

Indeterminate spinner only when total unknown. Determinate bar when progress measurable. Never fake 0→90→100 progress without semantics.

Long operation should be cancelable when backend supports consistent cancellation.

# Errors

Error structure:

- human-readable summary;
- affected resource/action;
- recovery action;
- optional technical details/copy diagnostics.

Examples:

`Couldn’t export “Poster 3” as TIFF because the selected CMYK profile is unavailable.`

Actions: Choose Profile, Export as RGB, Cancel.

# Warnings

Warnings explain risk but allow continuation: unsupported effect rasterization, missing bleed, overwrite conflict. Use warning semantic color + icon + text.

# Empty states

Every major panel defines empty state:

Layers no document; Assets no results; History new document; Data Merge no source; Export no targets; Plugins none installed. Empty state provides next action, not decorative illustration alone.

# Loading/skeleton

Use skeleton for content whose layout is known and takes noticeable time; use spinner for compact unknown task. Avoid skeletons in tiny property panels where a disabled/loading row is clearer.

# Undo feedback

Undo/Redo action name appears in Edit menu and optionally status hint (`Undo Move`). Destructive but reversible operations should lean on reliable undo rather than repeated confirmation dialogs.