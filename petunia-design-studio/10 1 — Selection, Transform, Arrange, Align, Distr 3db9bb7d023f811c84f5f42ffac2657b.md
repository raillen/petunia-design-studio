# 10.1 — Selection, Transform, Arrange, Align, Distribution & Snapping

# Selection model

Selection is session state, never canonical document content. Supports object selection, direct/node selection and context-specific sub-selection. Ordered selection preserves primary/key object for alignment and Properties semantics.

# Hit testing

Coarse spatial query → z/order filtering → exact hit evaluation. Selection tolerance is screen-space aware. Click cycling through overlapping objects is deterministic; modifiers add/remove/toggle according to keymap policy.

# Marquee

Directional/contain-vs-intersect policy is configurable or clearly defined. Auto-pan at viewport edges. Locked/hidden objects excluded unless explicit mode.

# Transform

Move, scale, rotate, skew with pivot/origin. Live preview occurs in transaction; commit creates one history entry. Modifier constraints are semantic actions (constrain proportions, from-center, duplicate-drag, angle snap) and remappable.

# Multi-selection

Transform applies around selection bounds or individual origins depending mode. Mixed transforms shown as Mixed; numeric field edits define exact operation rather than guessing.

# Align/distribute

Targets: selection bounds, key object, Surface, margins, last selected. Distribution supports equal gaps and spacing value. Algorithms document edge/bounds definitions for rotated objects.

# Snapping

Candidate families: grid, guides, object bounds, centers, nodes, path points, intersections, baselines, margins/columns, equal spacing. Candidate scoring uses screen distance + priority; hysteresis prevents flicker. UI overlay shows source/target and measurement.

# Arrange

Forward/back/front/back respects structural parent; crossing clipping/group boundaries requires explicit command. Reparenting is separate from z-order.

# Tests

Overlapping objects, rotated/skewed bounds, nested groups, locked layers, tiny zoom/huge zoom, mixed units, modifier changes mid-drag, cancel exactness, snapping conflict fixtures.

# Implementation contract — V1

## Scope status

**V1_REQUIRED:** object selection, direct/node selection, marquee selection, move/scale/rotate/skew, pivot/origin control, duplicate-drag, arrange/z-order, align/distribute, configurable snapping families and deterministic overlap cycling. Any additional selection paradigm requires an explicit scope status/ADR.

## Selection state machine

`Idle → HoverCandidate → PressCandidate → DragThreshold → ClickSelect | MarqueeSelecting | Transforming → Commit | Cancel`.

- A pointer press must not mutate selection until click-vs-drag intent is resolved according to the input threshold policy.
- `Esc`/cancel restores the exact pre-interaction selection and transform transaction state.
- Selection order is stable: the last intentionally selected eligible object becomes the **key/primary object** unless the user explicitly chooses another key object.
- Hidden objects are never hit-testable. Locked objects may be inspectable/selectable only through an explicit preference/mode; they are never transformable while locked.

## Coordinate-space contract

Every transform operation declares its space explicitly: document, parent/local, Surface or selection-oriented coordinates. Properties UI and MCP must report the active space instead of silently mixing local and world values.

- Move numeric edits set or offset position according to the selected Properties mode.
- Multi-selection size edits must expose whether the operation scales the group bounds or writes a shared per-object value; no ambiguous implicit behavior.
- Rotation is normalized for presentation only; canonical transforms must not be destructively rewritten merely to display a preferred angle range.

## Transform transaction

On pointer-down over a transform handle, capture: selected IDs, original transforms, selection bounds, pivot, active coordinate space, modifier/action state and document revision. Preview is derived from this snapshot. Commit produces one transaction/undo entry and one coherent ChangeSet; cancel applies no canonical mutation.

If selected objects are externally changed during a long transform, the commit must revalidate revision/IDs and either safely rebase the well-defined operation or fail with a stale-context diagnostic rather than overwrite newer state.

## Duplicate-drag

Duplicate-drag is one semantic operation: clone eligible selected objects/resources as needed, transform the clones, then commit atomically. Cancel removes all provisional clones. History must not contain a separate hidden duplicate entry followed by a move entry unless the command model explicitly exposes that behavior.

## Align/distribute semantics

Alignment operates on a declared reference: selection, key object, Surface, margins or explicit reference geometry. Rotated/skewed objects use evaluated world bounds unless a command explicitly requests local geometry anchors. Equal-gap distribution sorts objects deterministically along the chosen axis and must define whether strokes/effects contribute to visual bounds.

## Snapping lifecycle

Snapping is a query service, not tool-owned ad hoc logic. Each query supplies moving geometry, viewport scale, enabled families, constraints and exclusions. Result includes source/target semantic IDs, snap type, document-space point/axis, screen-space score and overlay metadata.

Use hysteresis: once a candidate is acquired, retain it until the pointer exceeds a release threshold or a substantially better candidate wins. This avoids flicker at intersections. Candidate ordering must be deterministic for equal scores.

## Keyboard/modifier policy

Tools consume semantic intents such as `Constrain`, `FromCenter`, `Duplicate`, `DisableSnapTemporarily` and `FineAdjust`; they do **not** hard-code Ctrl/Alt/Shift inside domain logic. The shell/keymap maps physical keys to these intents.

## Automation and plugins

Expose selection summaries and transform/alignment operations through Actions/Property schemas so MCP and Lua plugins reuse the same semantics. Pixel-coordinate mouse emulation is not the canonical automation path.

## Accessibility/usability

Keyboard-only users must be able to select next/previous eligible object, enter transform values, align/distribute and move by nudge increments. Status/Properties should expose why an object cannot be transformed (locked, read-only provider, missing capability) rather than silently ignore input.

## Required conformance tests

Add deterministic fixtures for coordinate spaces, key-object alignment, duplicate-drag cancel, revision conflict during drag, selection cycling, transformed visual bounds, snap hysteresis, locked/hidden objects, keyboard-only flow and UI↔MCP semantic parity.