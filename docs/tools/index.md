# Tool catalog

Atomic reference for every interactive tool. All tools obey the same contract: gestures preview, `Up` commits via `Action → Command → DocumentMutator → ChangeSet`, and **one gesture = one undo**. Shortcuts below are the canonical app bindings (see [Manual](/manual/)).

## How to read this catalog

- **Gesture** — pointer/keyboard flow (Down / Move / Up, modifiers).
- **Commits via** — the Command that lands in history.
- **Overlay** — live canvas feedback during the gesture.
- **Status** — `Wired` (live), `Disabled` (declared, disabled with reason), `Absent` (not implemented, never faked).

## Design persona

| Tool | Shortcut | Gesture | Commits via | Status |
| ---- | -------- | ------- | ----------- | ------ |
| Select | ++v++ | Click/marquee; ++shift++ toggle; ++alt++-drag duplicates | `SetBounds` / `CreateShapeObject` | Wired |
| Node | ++a++ | Grab anchor within 12 px/zoom, drag vertex | `SetShape` (batched) | Wired |
| PointTransform | — | Move sticky pivot, scale/rotate about it | `SetBounds` | Disabled (folded into Transform HUD) |
| Pen | ++p++ | Click anchors, drag symmetric tangents; click start ≤ 10 px closes | `Create path` | Wired |
| Pencil | ++n++ | Freehand ≥ 2 px, smoothed + fitted on release | `Create path` | Wired |
| Corner | — | Drag from selection center; ++shift++ all four corners | live `ModifierKind` | Wired |
| Contour | — | Radial drag = live offset; collapse preserves previous | live `ModifierKind` | Wired |
| Perspective | — | Drag one quad corner; degenerate quads refuse | live `ModifierKind` (homography) | Wired |
| Knife | — | Drag = cut line sampled into real pieces (closed stays closed) | batch `submit_all` | Wired |
| Scissors | — | Click = split open stroke in two / open loop in one | `split_path_at_point` | Wired |
| Rectangle / Ellipse / Polygon / Star | ++m++ | Drag + snap; < 2 px → 100×100 default; ++shift++ 1:1 | `create_shape_commands` | Wired |
| ShapeBuilder | — | Click creates, ++alt++ subtracts, drag merges regions | `ApplyBoolean` | Wired |
| SmartFill (VectorFloodFill) | — | Click bounded face (`frame − union`); unbounded = NoOp | `CreateObject` | Wired |
| ArtisticText | ++t++ | Click = 160×32 headline; drag = sized headline; click on path = on-path text | `CreateObject` + `SetBounds` + `SetShape` + `SetFill` | Wired |
| FrameText | ++t++ | Drag rect (min 20×20); columns/flow are Post-V1 | `frame_text()` | Wired |
| Gradient (fill) | ++g++ | Drag vector; double-click line adds stop, double-click stop removes (min 2) | `SetAppearance` | Wired |
| Transparency | — | Drag vector; default opaque→transparent; replaces legacy opacity proxy | `SetModifiers` | Wired |
| ColorPicker | ++i++ | Click topmost unlocked object; samples fill + stroke | `SetAppearance` per target | Wired |
| StylePicker | — | Click samples the whole AppearanceStack | `SetAppearance` | Wired |
| Artboard | ++a++ | Drag + snap; < 10 px → 1280×720; presets are future | `create_artboard_commands` | Wired |
| Measure | — | Transient; never mutates; drag = distance/delta/angle, area mode on drag-rect | — (readout only) | Wired |
| Zoom | ++z++ | Click/drag zooms to cursor focus | `CameraAction::Zoom` | Wired |
| Hand | ++space++ | Drag pans | `CameraAction::Pan` | Wired |

## Photo persona

| Tool | Shortcut | Gesture | Commits via | Status |
| ---- | -------- | ------- | ----------- | ------ |
| MarqueeRect | — | Drag rect; ++shift++/++alt++ = Add/Subtract modes | `RasterSelection` commit | Wired |
| MarqueeEllipse | — | Drag ellipse; same modes | `RasterSelection` commit | Wired |
| Lasso | — | Freehand loop; auto-close | `RasterSelection` commit | Wired |
| SelectionBrush | — | Paint diameter/hardness; edge snap | `RasterSelection` commit | Disabled (needs pixel layers) |
| FloodSelect | — | Click + tolerance; contiguous preview | `RasterSelection` commit | Disabled (needs pixel layers) |
| PixelPaintBrush | ++b++ | Dabs recorded; discarded without pixel layer | — | Disabled (pixel layers Post-V1) |
| PixelEraser | ++e++ | Same engine as paint | — | Disabled (pixel layers Post-V1) |
| Crop | ++c++ | With selection = vector crop modifier; bare = surface crop (min 10 px) | `SetSurfaceGeometry` | Wired |

## Rules every tool follows

1. **Preview, then commit** — `Down`/`Move` never write history; `Up` commits once.
2. **Never silent-lossy** — degenerate fragments (< 1 pt) drop loudly; micro-deletion is forbidden.
3. **Implicit conversion only inside the gesture that needs it** (e.g. knife auto-converts parametric shapes in the same batch).
4. **Locked/invisible never hit** — `visible && !locked` filter everywhere.

Full per-tool UX contracts live in the canonical atlases; developer-facing gesture plumbing is in [Architecture](/developers/architecture).
