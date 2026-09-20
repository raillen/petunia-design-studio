# 10.3 — Shapes, Boolean Operations, Shape Builder, Offset, Outline & Compound Paths

# Parametric shapes

Rectangle, rounded rectangle, ellipse, polygon and star are **V1_REQUIRED** parametric shapes with editable parameters until Convert to Curves. Additional specialty primitives are **POST_V1_CANDIDATE** unless promoted by an accepted scope decision. Shape-specific handles are overlays, not new object types per UI state.

# Live Boolean

Stores operation + operand references/order and produces evaluated geometry. Operands remain independently editable. Options define fill-rule and transform evaluation semantics. Missing operand yields diagnostic placeholder without data loss.

# Bake Boolean

Creates canonical path result and applies documented style inheritance; original operands replaced/retained based command variant. One undo restores live structure.

# Pathfinder

Union, subtract, intersect, xor/exclude, divide. Multi-operand ordering defined, especially subtract/divide. Results preserve provenance when engine provides it for style/shape-builder workflows.

# Shape Builder

Build arrangement of selected contours → enumerate bounded regions → hover preview → click/drag add/subtract regions → commit to paths/live region group according to mode. Region IDs exist only within transaction/evaluation, not durable object IDs unless committed.

# Smart Fill

Click enclosed region derived from visible/selected geometry and create new filled path. Gap tolerance must be explicit because it changes topology.

# Compound paths

Compound path is one object with multiple subpaths and fill rule; release/split commands preserve order/styles.

# Offset Path

Positive/negative offset with join/cap/miter options. Detect collapse/self-intersection; preview warnings rather than NaNs. Live Offset modifier preferred plus Expand variant.

# Outline Stroke

Converts semantic stroke to filled path; variable width/dashes/brush behavior documented per stroke type.

# Tests

Coincident edges, holes, nested fill rules, very thin offsets, star degeneracy, large operand counts, live operand deletion/duplication, shape-builder drag across adjacent regions.

# Implementation contract — V1

## Scope status

**V1_REQUIRED:** rectangle/rounded rectangle, ellipse, polygon, star; Convert to Curves; Live Boolean and Bake/Expand; union/subtract/intersect/xor; compound paths; Offset Path; Outline Stroke; Shape Builder baseline; Smart Fill baseline where the geometry engine can produce bounded regions reliably.

**POST_V1_CANDIDATE:** additional specialty parametric primitives unless promoted by an accepted scope decision.

## Parametric-shape contract

Each shape kind has a versioned parameter schema and deterministic evaluation function. Canonical parameters remain semantic (for example rectangle size/corner radii, polygon side count, star inner/outer ratio) rather than storing the evaluated Bézier path as truth. Editing through Properties, canvas handles, MCP and Lua must produce the same parameter mutation.

Invalid parameter combinations are clamped or rejected according to the descriptor; clamping must be visible and deterministic. Convert to Curves is an explicit destructive-to-parametrics command and one undo restores the parametric object.

## Live Boolean ownership

A Live Boolean stores stable operand IDs, ordered operation semantics and options. It does not steal ownership of operand geometry. Operand transforms/styles remain editable according to the group contract. Cycles are rejected at command validation. Missing operands produce a preserved diagnostic state rather than deleting remaining references.

## Style inheritance

Boolean/pathfinder commands must define style selection explicitly. Default V1 policy should be deterministic and documented per command (for example topmost/key operand), with an option to retain per-region appearance only when provenance can represent it safely. Agents must not invent style rules from visual order ad hoc.

## Shape Builder state machine

`Idle → ArrangementPrepared → HoverRegion → AccumulatingAdd/Subtract → PreviewResult → Commit | Cancel`.

Arrangement data and transient region IDs are scoped to the transaction/revision. If source geometry changes, invalidate and rebuild rather than applying old region IDs. Dragging across regions must use deterministic traversal and avoid double-applying the same region.

## Smart Fill

Smart Fill queries a bounded arrangement in a declared source scope (selected objects, current container or visible eligible geometry according to tool mode). Gap tolerance is a user-visible semantic parameter because it changes topology. Result is a new canonical path/style command; source geometry is not modified.

## Offset/outline semantics

Offset sign, join, cap, miter and self-intersection handling are explicit. Preview may warn when inward offsets collapse regions. `Outline Stroke` must evaluate the complete semantic stroke — alignment, dashes and variable width where supported — before producing filled paths. Unsupported brush semantics must return an explicit limitation rather than approximate silently.

## Determinism and performance

Interactive previews may use reduced evaluation quality only if final commit reevaluates from canonical inputs with final tolerances. Large boolean/shape-builder work may use a cancelable background preview, but committing must revalidate the source revision.

## Automation/plugins

All operations expose Actions/typed schemas; Live Boolean and Offset are discoverable as semantic effects/modifiers rather than opaque UI-only constructs. Lua/MCP receive stable object IDs and operation enums, never transient arrangement region IDs beyond a scoped transaction token.

## Required tests

Add parameter-schema roundtrip, convert-to-curves undo, live-boolean cycle rejection, missing operand recovery, deterministic style inheritance, stale arrangement revision, Smart Fill gap tolerance, inward-offset collapse, outline dashes/variable width, cancellation and headless UI↔MCP parity.