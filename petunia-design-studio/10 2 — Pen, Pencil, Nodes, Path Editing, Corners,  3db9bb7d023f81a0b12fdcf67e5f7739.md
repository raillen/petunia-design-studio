# 10.2 — Pen, Pencil, Nodes, Path Editing, Corners, Knife & Scissors

# Pen state machine

Idle → first anchor → active subpath → segment preview → anchor commit → close/open finish/cancel. Pointer drag creates handles; modifiers constrain handles/angle, break handles, temporarily switch node semantics according to keymap.

# Node types

Cusp, smooth and symmetric are editing constraints/intent metadata where useful, not separate geometry primitives. Conversion is undoable.

# Direct selection

Select individual nodes/segments, box-select nodes, shift toggle, delete with topology-aware result, move with snapping, transform node subset.

# Segment editing

Drag curve segment adjusts adjacent handles using documented curve-edit algorithm. Add node places parameter-aware split preserving curve shape. Remove node has modes: simple delete vs smart delete approximating shape under tolerance.

# Join/Break/Close

Join endpoint selection rules, distance/snapping, tangent continuity options. Break creates endpoints while preserving geometry. Close adds explicit segment.

# Pencil/freehand

Collect samples → smoothing/simplification preview → final Bézier fitting under tolerance. Stabilizer optional; pressure may map to width profile when brush/stroke supports it.

# Corner tool

Select eligible nodes, drag radius, modes round/chamfer/concave if supported. Live corner remains parametric when feasible; Bake produces path.

# Knife/Scissors

Scissors splits at path hit. Knife cuts across one/many paths using intersections; closed-fill result policy defined. No silent deletion of tiny pieces without threshold setting.

# Precision

Handle/node HUD supports coordinates, angle, length and keyboard entry. All operations respect document f64 and operation-specific tolerance.

# Tests

Self-intersections, zero-length handles, overlapping nodes, near-closed endpoints, extreme scales, smart-delete error bounds, cancel/undo, save/reopen live corners.

# Implementation contract — V1

## Scope status

**V1_REQUIRED:** Pen, Pencil/freehand, Node/Direct Selection, add/remove node, cusp/smooth/symmetric conversion, join/break/close path, segment editing, Corner tool baseline, Scissors and Knife baseline. Experimental corner variants or advanced calligraphic behaviors must be tagged separately.

## Pen tool interaction model

The Pen tool keeps provisional state outside the canonical document until an anchor/segment is committed. State includes current subpath, last anchor, provisional handle, snap result, constraint intent and whether the interaction is extending an existing path.

Canonical phases:

1. `Idle`;
2. `Started` after first anchor;
3. `SegmentPreview` while pointer moves;
4. `HandleAdjust` while dragging;
5. `AnchorCommitted`;
6. `ClosePreview` when hovering eligible start node;
7. `Finished` or `Cancelled`.

`Esc` first cancels the current provisional segment; a second cancel/Finish action ends the current open path according to shell policy. The exact physical shortcut remains keymap-owned.

## Existing-path continuation

Starting near an eligible endpoint may extend that path only after an explicit hit target is resolved. The tool must visually distinguish “create new path” from “extend path” and never merge paths merely because two endpoints are close.

## Handle/node semantics

Smooth and symmetric states are constraints over handles. Breaking one handle is an explicit interaction. Node conversion must preserve anchor position and produce deterministic handle positions; no hidden smoothing on unrelated edits.

## Node deletion

Provide two semantic commands:

- **Delete Node:** removes the node and reconnects topology using the basic documented rule;
- **Smart Delete:** attempts to preserve visual shape within a configurable/tolerance-bounded error.

Smart Delete must report failure/large deviation rather than silently produce a materially different curve.

## Pencil pipeline

`raw samples → device normalization → optional stabilization → resampling → simplification → Bézier fitting → optional pressure/width mapping → preview → commit`.

Sampling timestamps/pressure are transient unless needed by the committed stroke/brush semantics. Fitting tolerance is viewport/geometry aware but deterministic from recorded semantic inputs. Cancellation discards the provisional fit.

## Knife and Scissors

Scissors acts on an exact path hit and creates topology splits without deleting geometry. Knife uses an explicit cut path and intersection set. Operations across groups/clips must state their scope; they may not silently escape the selected structural context. Tiny fragments are preserved unless the user invokes an explicit cleanup threshold/action.

## Corner tool

V1 baseline supports live parametric corner metadata for eligible nodes where the source object/path representation can preserve it; otherwise the tool may preview a baked geometric result and must clearly expose that distinction. Radius clamping at self-intersection/segment limits is deterministic and visible.

## Tool transaction rules

Every gesture has `begin/update/commit/cancel`; previews never create separate undo history. Switching tools with a provisional path must invoke an explicit Finish/Cancel policy, not leave hidden partial transactions.

## Automation/plugin surface

MCP/Lua should prefer semantic operations such as create path, add node, set handles, split path and convert node type. Plugin tools receive normalized pointer phases and may request safe overlay descriptors; they do not receive raw GPUI events or geometry-engine internals.

## Required tests

In addition to existing cases, test path extension ambiguity, cancellation at every state, modifier-intent remapping, smart-delete error threshold, pencil deterministic replay, pressure mapping, knife through compound paths, corner-radius clamping, document revision conflict during tool gesture and UI↔MCP parity.