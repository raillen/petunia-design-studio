# 09.31 — Universal Non-Destructive Editing: EffectChain, Live Modifiers, Explicit Bake

<aside>

**Status:** `ACCEPTED_V1`. Implements the `AGENTS.md` / `01` / `02` / `03` doctrine
(default non-destructive via typed ordered EffectChain; Bake/Expand/Rasterize/
Convert-to-Curves as explicit user operations only). First live modifier:
`ContourOffset`. Supersedes: nothing (first EffectChain ADR).

</aside>

# Decision question

How do all edits — vector and raster, present and future — stay non-destructive
without freezing tool velocity?

# Context

Every transformative edit rewrote source geometry: `offset_path` inflated bounds
and scaled vertices in place, corner edits collapsed four radii into one, sculpt
replaced paths without a trace. Undo restored state, but undo is a backstop, not
a live model. The architecture mandates a typed ordered EffectChain, yet no
`EffectChain`, `LiveModifier`, or equivalent existed anywhere in `crates/`.

# Constraints

- `AGENTS.md:9`, `01:75`, `02:29`, `03:31`: non-destructive default; Bake/Expand/
  Rasterize/Convert-to-Curves explicit-only.
- F-01 (one gesture, one undo), F-21 (explicit flatten tolerance, curve-exact
  booleans POST_V1), `NATIVE_SCHEMA_VERSION = 1` files must keep loading.
- No lateral feature-to-feature storage access; UI receives DTOs.

# Options

1. **Foundation now, Contour first (chosen).** Typed `ModifierKind` + ordered
   chain on `DocumentObject`, base-vs-evaluated read doctrine, true offset math,
   `SetModifiers`/`BakeContour` commands, Contour tool migrated, per-corner
   parametric rectangles with render fix.
2. **Hybrid (rejected for now).** ADR now, code later with "preserve source"
   discipline. Rejected: each new batch would grow the migration debt the
   correction was made to stop.
3. **Guideline-only (rejected).** ADR plus rules, retrofit when painful.
   Rejected: defers the correction itself.

# Decision

- `petunia_design_document::modifiers`: `ModifierItem { id, kind, enabled }`,
  `ModifierKind::ContourOffset { distance, join, cap }`; new kinds extend the
  enum, never a generic destructive operation.
- `DocumentObject.modifiers: Vec<ModifierItem>` (`#[serde(default)]`: v1 files
  load unchanged, no migration).
- **Base vs evaluated doctrine:**

  | Reader | Reads | Why |
  |---|---|---|
  | Node/Pen/Pencil edits, `convert_to_curves` | `to_path()` (base) | tools edit the source |
  | Render, hit-test, selection bounds, booleans, SVG/PDF export, canvas preview | `evaluated_path()` / `evaluated_bounds()` | everyone sees the same live geometry |
  | `BakeContour` (explicit only) | evaluated → base `Path`, clears contour entries | the one sanctioned freeze |

- True offset math: expansion follows the stroked outer edge (curves preserved);
  insetting erodes through the offset engine (despiked; curves flatten at 0.25pt
  per F-21). Collapse keeps the previous result instead of destroying it.
- `Change::ModifiersChanged` flows through mutator revert and history replay.
  Legacy `OffsetPath` now upserts the live modifier (same call sites, no
  destruction). `SetModifiers` commits whole chains in one undo entry.
- Corner tool: per-corner parametric radii on rectangles (`[TL, TR, BR, BL]`),
  `rect_corners` render honors all four (previously only index 0 rendered),
  physical clamp stays, one undo entry. Non-rectangles are left alone —
  conversion stays an explicit user action.
- Contour tool: drag commits one live offset per selected object in one undo
  entry, with pending-outline preview; `Bake Contour` button mirrors Bake Corners.

# Consequences

- Geometry: `geometry::offset{,_path, OffsetJoin, OffsetCap}`, `GPath::rect_corners`.
- Document: `modifiers` module, `evaluated_path/bounds`, `to_path` documents base;
  `set_modifiers`, `set_contour_offset`, `bake_contour`; `OffsetPath` rerouted.
- Application: `SetModifiers`, `BakeContour`; booleans read evaluated; redo replays.
- Shell/ UI: `set_contour_offset`/`bake_contour` bridge calls; selection and
  properties show evaluated bounds; canvas/SVG/PDF draw evaluated outlines;
  Bake Contour button next to Bake Corners.
- 51-load-bearing-doc audit: doctrine pages already mandate this; no page
  contradicts the decision.

# Follow-ups (not blockers)

- `CornerType` per corner (chamfer/concave): needs model + builder work.
- Curve-exact inset: expansion preserves curves, inset flattens (F-21). Unify POST_V1.
- Live drag preview layer (modifiers preview without commit) for all tools.
- Raster ND (adjustment layers, live filters, masks) when pixel tools ship.

# Revisit trigger

A second modifier kind must generalize entry identity/ordering UI and confirm
evaluation cost stays linear. Only measured evaluation bottlenecks or a format
migration requirement may supersede this ADR, explicitly.
