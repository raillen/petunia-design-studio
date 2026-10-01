# 08.19 — Microcopy, Labels, Naming, States, Units & Interface Language

# Voice

Aubrieta UI language is concise, neutral, helpful and professional. Avoid playful wording in destructive/error contexts and avoid technical implementation jargon when a user-facing domain term exists.

# Naming principles

- Use verbs for actions: `Export`, `Convert to Curves`, `Relink`.
- Use nouns for destinations/panels: `Layers`, `Properties`, `History`.
- Use sentence case in menus/dialogs unless platform convention requires otherwise.
- Preserve recognized creative-industry terminology when it reduces cognitive load: Fill, Stroke, Opacity, Blend Mode, Pathfinder, Curves, Levels.
- Do not rename standard concepts merely to sound unique.

# Ellipsis

Use `…` in action title when activation requires additional input before command completes: `Export…`, `Preferences…`, `Relink…`. Do not add ellipsis merely because a window opens if action is conceptually immediate and platform convention says otherwise; consistency matters more than decoration.

# Labels

Prefer short stable labels. Units should be visible adjacent to values or encoded in field suffix. Avoid placeholder-only labels for important settings; labels remain visible after value entry.

# Tooltips

Pattern:

**Tool Name** — shortcut

One concise explanatory sentence when needed.

Optional: `Learn more` topic.

Example:

**Node Tool — A**

Edit path nodes and Bézier handles.

# Error language

Structure:

1. What happened.
2. What is affected.
3. What user can do.

Good: `Couldn’t relink “cover-photo.tif” because the selected file has a different resource type.`

Bad: `Error 0x12: invalid asset.`

Technical details belong behind Details/Copy Diagnostics.

# Warning language

Describe consequence, not fear. `This effect isn’t supported by SVG and will be rasterized during export.` Then offer Preview/Continue/Cancel if choice matters.

# Destructive actions

Button names state the destruction: Delete, Remove Link, Discard Recovery. Avoid generic `OK` for destructive confirmation.

# Success language

Most successful direct manipulations need no toast. Toast only when result is not otherwise visible: `Preset saved`, `Export complete`, `Copied SVG`.

# Empty states

Pattern:

- concise state;
- one next-step action.

Example: `No data source attached.` + `Attach Data Source…`

Avoid paragraphs of onboarding inside utility panels.

# Mixed values

Use `Mixed` or em dash in a field according to control type. Accessibility label states `Mixed values`. Do not silently display one selected object's value as though universal.

# Boolean states

Use direct positive labels: `Snap to Grid`, not `Disable no snapping`. Avoid double negatives.

# Units

Supported unit labels may include px, pt, mm, cm, in, %, ° and document-specific units. Unit parser normalizes input but preserves user's chosen document display unit where possible.

# Numeric formatting

- coordinates/dimensions: precision based on unit/zoom context, not unlimited decimals;
- percentages: integer by default, decimals when entered/required;
- angle: degree symbol;
- color channels: model-appropriate range visible/consistent;
- use locale decimal separator in UI while serialization remains canonical.

# Menu/action consistency

The same action has the same base name in menu, command palette, shortcut editor and tooltip unless context needs grammatical adaptation. Avoid four names for the same operation.

# Capitalization examples

Panel: `Data Merge`

Action: `Generate Data Merge…`

Field: `Rendering intent`

Section: `Color management`

Tooltip sentence: `Show colors through the selected proof profile.`

# Terminology registry

Maintain a localization-facing glossary for canonical words: Surface, Artboard/Page presentation, Layer, Group, Mask, Clip, Symbol, Asset, Adjustment, Effect, Persona, Workspace, Profile, Swatch, Data Source, Binding, Record.

# Status bar hints

Hints are short imperative or stateful phrases: `Drag to create a rectangle · Shift constrains proportions`. They disappear/change with tool context and are not essential instructions.

# Accessibility copy

Accessible names describe outcome, not icon appearance: `Toggle layer visibility`, not `Eye icon`. When visual label already clear, avoid redundant verbose screen-reader text.

### Object-editor implementation — 2026-10-01

Scope: **Milestone Required (MVP)**. Explicit stable session/object drafts now
cover Rename, multiline content and exact local placement with points/degrees.
Cancel/conflict/invalid input do not publish. Typography/path and exact rotation
are preserved; multiselection, direct canvas text and external a11y remain open.
See [ADR-008](../docs/developers/adr/ADR-008-object-edit-drafts.md) and
[implementation/remaining-work/evidence](../docs/developers/uiux-object-edits.md).
