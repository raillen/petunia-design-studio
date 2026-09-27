# 10.3.1 — Shape and Handle Reference Dossier

**Status:** OPEN · research runbook · **Started:** 2026-09-25

## Purpose

This dossier records the observed behaviour of shape creation and transform
handles in the market references, so `10.3` (parametric shapes) and `10.1`
(move/scale/rotate/skew with pivot) can be implemented against evidence rather
than assumption.

## Evidence level of this document

**Every question below is UNVERIFIED.** The research run that was supposed to fill
this dossier failed on 2026-09-25 with HTTP 429 (weekly plan quota, resets
2026-09-30). No external source was reached: not the Affinity documentation, not
the Inkscape source, not the Graphite or Krita documentation.

Nothing in this file may be cited as market behaviour until its `Result` field
is replaced by an observed answer with a URL. The contracts already accepted in
`08 22`, `08 23`, `10 1` and `10 3` remain the authority for what we build now;
this dossier records what still needs confirmation.

Levels used below: `OBSERVED` (read in a primary source), `SOURCE-VERIFIED`
(confirmed in open source code), `INFERRED` (reasoned, not confirmed),
`UNVERIFIED` (not reachable).

## Q1 — Shape creation

| # | Question | Target source | Result |
| --- | --- | --- | --- |
| Q1.1 | Does a click without a drag create a shape, and at what default size? | `affinity.help` shape tool page; CorelDRAW Rectangle tool | UNVERIFIED |
| Q1.2 | During creation, what does Shift constrain: aspect ratio, angle, or axis alignment? | `affinity.help` `tools_move`; Inkscape tool modifiers | UNVERIFIED |
| Q1.3 | What does Alt do during creation — from-centre, duplicate, or subtract? | `affinity.help`; Shape Builder modifier table | UNVERIFIED |
| Q1.4 | Is a created shape immediately selected with its handles active? | `affinity.help`; observed in product | UNVERIFIED |
| Q1.5 | Does the shape tool keep an "active parameter" (side count, star points) for the next shape, or reset it? | Affinity Shape Tool panel | UNVERIFIED |
| Q1.6 | After creation, where are the shape parameters edited — panel, context bar, or both? | `affinity.help` panels; Geometry panel | UNVERIFIED |
| Q1.7 | For polygon/star, are the parameters editable by dragging a handle on the canvas itself? | `affinity.help` shape tool | UNVERIFIED |

## Q2 — Transform handles

| # | Question | Target source | Result |
| --- | --- | --- | --- |
| Q2.1 | What is the canonical handle set on a selected object? | `affinity.help` `tools_move`; Inkscape `src/ui/selhandlers.cpp` | UNVERIFIED |
| Q2.2 | How is the rotation handle placed, and how far from the frame? | `affinity.help` `tools_move` | UNVERIFIED |
| Q2.3 | On a corner handle, what is the exact behaviour with Shift, with Alt, and with neither? | `affinity.help`; CorelDRAW property bar | UNVERIFIED |
| Q2.4 | Does the handle snap to guides, pixels, or angle increments while dragging? | `affinity.help` `tools_snap`; Inkscape snapping | UNVERIFIED |
| Q2.5 | Does proportional scaling have a distinct mode, and how is it toggled? | `affinity.help`; CorelDRAW | UNVERIFIED |
| Q2.6 | Is there a centre handle separate from the four corner handles, and what does it drive? | `affinity.help` `tools_pointTransform` | UNVERIFIED |
| Q2.7 | Is rotation constrained by an increment (15°/45°)? Under which modifier? | `affinity.help`; Inkscape rotation snapping | UNVERIFIED |
| Q2.8 | When dragging a handle on an already-rotated object, does the resize happen in screen space or in object-local space? | Inkscape `selhandlers.cpp`; Affinity behaviour | UNVERIFIED |
| Q2.9 | How are handles visually distinguished — shape, size, fill, hover state? | `affinity.help`; `08 23` already states the rule | UNVERIFIED |
| Q2.10 | Is there a numeric Geometry panel that mirrors handle state? | `affinity.help` Geometry panel | UNVERIFIED |

## Q3 — Corner radius

| # | Question | Target source | Result |
| --- | --- | --- | --- |
| Q3.1 | Are the four corner radii independent, or does editing one mirror the others? | `affinity.help` corner tool | UNVERIFIED |
| Q3.2 | Which modifier mirrors all four radii? | `affinity.help`; our own `contour.rs` uses `constrain` | UNVERIFIED |
| Q3.3 | Is a maximum radius clamped, and is the clamp visible while dragging? | `affinity.help`; our `09 5` numeric policy | UNVERIFIED |
| Q3.4 | Does corner radius survive non-uniform scaling, and how is it recomputed? | `affinity.help` | UNVERIFIED |

## Q4 — Skew and pivot

| # | Question | Target source | Result |
| --- | --- | --- | --- |
| Q4.1 | How is a pivot/origin placed and later moved? | `affinity.help` `tools_pointTransform` | UNVERIFIED |
| Q4.2 | Does the pivot persist on the object or reset per gesture? | `affinity.help` | UNVERIFIED |
| Q4.3 | Is skew a first-class transform, and how is it stored in the document? | Affinity; Inkscape | UNVERIFIED |

## Already accepted locally (not research output)

These are project decisions, not market findings. They are the current contract
while Q1–Q4 stay open.

- `10 3` line 5: rectangle, rounded rectangle, ellipse, polygon and star are
  V1_REQUIRED parametric shapes with editable parameters until Convert to Curves.
  Shape-specific handles are **overlays**, not new object types.
- `10 3` line 53: parameters stay semantic (size, corner radii, polygon side
  count, star inner/outer ratio) rather than storing the evaluated Bézier path as
  truth. Properties, canvas handles, MCP and Lua must produce the same mutation.
- `10 3` line 55: invalid parameter combinations are clamped or rejected
  deterministically, and the clamp must be visible.
- `10 1` line 17: move, scale, rotate, skew with pivot/origin. Modifiers are
  semantic actions (constrain proportions, from-centre, duplicate-drag, angle
  snap) and are remappable.
- `08 23` line 56: handles encode role by shape + cursor + optional colour,
  never colour alone. Roles include transform, rotation, node, Bézier handle,
  corner radius, contour offset, gradient stop, transparency stop, crop, width
  point, text-flow link and guide.
- `08 23` line 58: pointer hit area scales separately from the visible glyph.
- `08 33`: specialty primitives (Triangle, Cog, Heart, Donut…) are
  POST_V1_CANDIDATE. *"Their existence in Affinity is not an argument to copy
  them all."*

## Known divergences between our behaviour and the unverified market model

Recorded so the eventual research has a concrete checklist. Each line is a
candidate divergence, **not** a confirmed defect.

| Area | Our behaviour today | Question |
| --- | --- | --- |
| Click threshold | `shape.rs:87-90` compares **document points** to `2.0` while the comment claims 2 px | Q1.1 |
| Constrain anchor | `shape.rs:91-96` anchors at `p0.min(p1)` without mirroring for up/left drags | Q1.2 |
| Alt semantics | `main.rs:303-309` maps Alt to both `from_center` and `duplicate` | Q1.3 |
| Shape parameters | `shape_factory.rs:50,55-63` hardcode sides=5, points=5, ratio=0.5 | Q1.5, Q1.7 |
| Handle appearance | all 9 handles render as identical 8 px squares | Q2.9 |
| Handle hit test | rotation handle sits above the frame; hit order is handle-then-border | Q2.2 |
| Resize space | screen-space handle drag onto a rotated object | Q2.8 |
| Skew | absent from the model (`SetBounds` carries only bounds + rotation) | Q4.3 |
| Pivot | `PointTransformTool` exists but has no preview and no test | Q4.1 |

## Execution plan (from 2026-09-30)

1. `affinity.help` — `tools_move`, `tools_pointTransform`, `tools_node`, the shape
   tool pages, and the Geometry panel. Record each page URL and answer Q1 and Q2.
2. Inkscape `src/ui/selhandlers.cpp` — read the real pivot/scale implementation
   to answer Q2.1, Q2.8 and Q4.3 at source level (`SOURCE-VERIFIED`).
3. Krita and Graphite documentation — cross-check LOD and node-handle behaviour.
4. Academic references for the transform algorithm with pivot remain outstanding;
   the workspace has no citation for them yet.

Each answer must record: source URL, access date, verbatim quote, and the
resulting level (`OBSERVED` or `SOURCE-VERIFIED`). Anything that cannot be
sourced stays `UNVERIFIED` and must not be implemented as if confirmed.
