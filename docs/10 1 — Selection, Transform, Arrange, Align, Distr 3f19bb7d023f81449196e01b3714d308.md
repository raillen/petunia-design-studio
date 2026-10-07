# 10.1 — Selection, Transform, Arrange, Align, Distribution & Snapping

# Selection model

SelectionState pertence à view/session, não ao documento. Mantém ordered selected ObjectIds, active/key object, sub-selection opcional e selection revision. Selecionar não cria history.

# Click selection

HitTestService C++ recebe point, viewport scale, filter/capabilities e preference tolerance. Retorna candidates ordenados por z-order, distance e semantic priority. Repeated click/cycle pode percorrer sobrepostos; modifier select-through abre list/cycle policy.

# Marquee

Drag em empty canvas inicia SelectionMarquee preview. Crossing vs containment configurable/modifier. Query usa spatial index. Visual rectangle é overlay, não object.

# Move

Pointer down em selected movable object abre TransformGesture. Core captura initial transforms + pivot. Pointer move calcula delta em document space, aplica axis constraint/snapping/duplicate preview. Pointer up cria TransformObjectsCommand com before/after transform semantic values.

# Resize

Bounding handles têm screen-space hit areas. Resize calcula local/object/world transforms corretamente. Shift aspect policy, Alt center, anchor point, rotated boxes e negative scaling são definidos. Numeric W/H uses same command path.

# Rotate

Rotation handle / R? as modifier not necessarily tool. Snap to angle increments when Shift. HUD angle; Transform panel mirrors live preview. Pivot draggable and persistent only if product decision says so; default pivot is derived center.

# Shear

Side/edge affordance or Transform panel. Shear stored in affine transform decomposition only when stable; otherwise canonical matrix with UI decomposition derived.

# Duplicate transform

Alt/Option drag clones selected objects at gesture start in staged transaction; cancel removes staged clones. Power Duplicate repeats last compatible transform+duplication delta.

# Arrange

Bring to Front, Forward, Backward, Send to Back operate within same parent/paint context. Cross-parent move is separate command to avoid hidden hierarchy changes.

# Align

Targets: Selection Bounds, First/Last selected, Key Object, Surface, Margin/Guide where valid. Align Left/Center/Right/Top/Middle/Bottom. Key object remains fixed.

# Distribution

Distribute centers/edges; Equal Spacing horizontal/vertical. If explicit spacing given, group may expand from anchor policy. UI preview can show measurements before commit.

# Flip

Horizontal/Vertical around selection bounds, key object or chosen anchor. Text/raster/vector remain editable; transform sign handled without rasterization.

# Nudge

Arrow = configured small step; Shift = large; Alt/Option optional duplicate-nudge. Repeated keydown coalesces into one history transaction within gesture window.

# Transform panel

X/Y/W/H/rotation/shear fields; anchor 3×3 grid; lock aspect; scale strokes/effects option. Mixed values show Mixed. Typing expression parsed safely and commits on Enter/focus rule.

# Snapping

SnapEngine candidates include geometry nodes/edges/centers, bbox, guides, grid, margins, text baselines and equal gaps. Each query returns snapped transform plus overlay descriptors. Threshold is screen-space aware.

# Commands

Select is view-state action. Document commands: TransformObjects, DuplicateAndTransform, ReorderObjects, AlignObjects, DistributeObjects, FlipObjects, SetPivot where persisted.

# Edge cases

Locked/hidden objects, nested transformed groups, symbols, live booleans, text frames, zero-size objects, extreme zoom, multi-Surface selections and mixed coordinate spaces.

# MCP/plugin

[object.select](http://object.select) (view-scoped), selection.transform dry-run/commit, arrange.*, align.*, distribute.* delegate to same command builders.

# Tests

Property tests for transform inverse/roundtrip, snap ranking, rotated bounds, duplicate cancel, nudge coalescing, mixed parent rejection and 10k-object marquee latency.