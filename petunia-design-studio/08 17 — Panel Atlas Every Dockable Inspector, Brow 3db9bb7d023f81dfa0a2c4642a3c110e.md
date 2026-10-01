# 08.17 — Panel Atlas: Every Dockable Inspector, Browser & Utility Panel

# Amendment 2026-09-21 — Petunia/Slint Panel Authority

Panel responsibilities remain canonical, but the presentation adapter is now Slint. Every panel must be built from Petunia Design System components and registered by stable PanelId. Default Design workspace grouping is refined by 08.34–08.36: Color/Swatches/Stroke/Appearance; Layers/Properties; Transform/Align/Navigator/History. Exact grouping is a workspace preset, not domain ownership.

No panel may maintain a second canonical copy of document state. Large trees/lists require incremental/virtualized presentation models and explicit loading/error/stale state.

# Purpose

This page enumerates the **canonical panel surface area** so implementation does not invent overlapping inspectors or leave major workflows without a home. Panels can share components, but their semantic responsibility must remain distinct.

| Panel | Personas | Primary responsibility | Core content |
| --- | --- | --- | --- |
| Layers | Design + Photo | canonical object hierarchy | tree, thumbnails, names, visibility, locks, masks/effects/data badges |
| Properties | Design + Photo | selection/tool inspector | transform, appearance, object-specific sections, mixed state |
| Color | Both | active semantic color editing | model selector, channels, swatch, opacity/alpha where applicable, profile hint |
| Swatches | Both | reusable colors/gradients | document/library swatches, groups, recent, search |
| Stroke | Design | outline appearance | width, alignment, cap, join, dash, markers, profile |
| Transform | Both | precision geometry | X/Y/W/H, rotation, skew, origin, flip, units |
| Align | Design | alignment/distribution | reference target, align, distribute, exact spacing |
| Pathfinder | Design | boolean operations | live/destructive operations, expand/bake |
| Typography | Design | character formatting | font, style, size, tracking, leading, OpenType, language |
| Paragraph | Design | paragraph formatting | alignment, indents, spacing, tabs, frame-related options |
| Text Styles | Design | reusable text formatting | character/paragraph styles, overrides, create/update |
| Pages / Surfaces | Design | artboard/page management | thumbnails, order, dimensions, names, bleed/export flags |
| Assets | Design + Photo | reusable resource browser | graphics, components, icons, textures, brushes, search/filter |
| Symbols | Design | symbol source/instances | symbol list, edit source, detach, overrides when supported |
| Libraries | Both | resource collections | library sources, sync/read-only status, categories |
| Styles | Both | layer/effect styles | style list/grid, apply, create, update, delete |
| History | Both | transaction history | V1 transactional undo/redo entries + current state marker; branching/time-travel is **Post-V1 Candidate** pending 09.3 semantics |
| Navigator | Both | document orientation/zoom | thumbnail, viewport rectangle, zoom/fit |
| Info | Both | cursor/document metrics | coordinates, dimensions, sampled color, object metadata |
| Data Merge | Design | variable-data binding/preview | source, fields, current record, bindings, preview/generate |
| Export | Both | quick output presets | target, preset, format summary, quick export |
| Histogram | Photo | tonal distribution | RGB/luma/channel histogram, clipping indicators |
| Adjustments | Photo | create nondestructive adjustment | adjustment catalog, favorites/recent optional |
| Channels | Photo | channel/mask operations | process channels, alpha/masks, visibility/load selection |
| Brushes | Photo | brush preset browser | categories, search, thumbnails, favorites/recent |
| Brush Settings | Photo | brush engine parameters | V1: shape, spacing, opacity/flow dynamics, pressure/tilt; **Post-V1 Candidate:** texture/scatter dynamics |
| Presets | Photo | image/adjustment presets | categories, preview, favorites, import/export |
| Masks | Photo | mask-specific inspector | density, feather, invert, refine, overlay |
| Resources / Links | Both | external resource management | linked/embedded, update, relink, missing state |
| Background Tasks | Both | long-running jobs | progress, cancel, retry, reveal result, error summary |
| Plugins | Both | plugin status/management | enabled, capability status, source/version, errors |

# Panel-specific behavior requirements

## Layers

Must support dense hierarchy, virtualized rendering, multi-selection, drag/reparent, rename, visibility/lock actions, thumbnails and object-state badges. Drag target visualization distinguishes sibling reorder, parenting and clipping/masking intent.

## Properties

Never becomes an unstructured dumping ground. Sections ordered by general→specific: Transform, Appearance, object-specific behavior, effects/export. Frequently used controls remain above advanced disclosures.

## Color / Swatches

Color edits semantic `aubrieta_color` values; UI family icons/colors do not leak into document model. Swatch application differentiates Fill vs Stroke through click/shortcut/context action. Spot/process distinction visually identifiable without relying solely on color.

## Assets

Header: search + filter + view mode. Content supports list/grid thumbnail sizes. Drag creates/place resource. Asset metadata available in inspector/context menu, not cluttering every tile.

## History

Entries are user-meaningful transactions (`Move 3 Objects`, `Change Fill`) rather than low-level mutations. Long operations may expand details only in diagnostic mode.

## Photo analysis panels

Histogram and other scopes update asynchronously and communicate stale/recomputing state. They must never block brush input.

# Panel empty/loading/error states

Each panel must define all three where applicable. For example:

- Assets empty search → `No assets match “…”` + Clear Filters;
- Data Merge no source → Attach Data Source;
- Links missing → warning row + Relink;
- Layers no document → concise noninteractive state;
- Plugins failed → error row + Details/Disable.

# Panel persistence

Persist tab ordering, size, section collapsed state and view mode in workspace/user settings. Do not persist transient selection/filter search in canonical document. Search retention may be session-specific.

### Object-editor implementation — 2026-10-01

Scope: **Milestone Required (MVP)**. Explicit stable session/object drafts now
cover Rename, multiline content and exact local placement with points/degrees.
Cancel/conflict/invalid input do not publish. Typography/path and exact rotation
are preserved; multiselection, direct canvas text and external a11y remain open.
See [ADR-008](../docs/developers/adr/ADR-008-object-edit-drafts.md) and
[implementation/remaining-work/evidence](../docs/developers/uiux-object-edits.md).
