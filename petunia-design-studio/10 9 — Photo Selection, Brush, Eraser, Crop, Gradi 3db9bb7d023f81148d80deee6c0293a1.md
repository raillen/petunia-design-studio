# 10.9 — Photo Selection, Brush, Eraser, Crop, Gradient, Clone & Heal Tools

# Raster selection tools

Rectangle/Ellipse/Lasso/Freehand selections modify session/document selection-mask state according to architecture decision. Modes New/Add/Subtract/Intersect; feather/antialias visible before/after commit. Select All/None/Invert/Grow/Shrink/Feather commands.

# Brush

Tool state: brush preset, size, hardness, opacity, flow, spacing, dynamics, blend/paint mode, smoothing, foreground color and target PixelLayer/mask. Pointer-down opens stroke transaction; samples feed brush engine; pointer-up commits one history entry.

# Eraser

Prefer brush engine with erase/alpha paint semantics rather than separate implementation. Behavior on layers without alpha must be explicit (add alpha, background color, reject based on document policy).

# Gradient

Raster gradient fills selected PixelLayer/selection/mask using GPU/reference engine. Design vector gradient tool remains separate action/tool contribution though UI icon may be related.

# Crop

Non-destructive document/Surface crop where possible vs destructive PixelLayer crop command. Photo Persona must make scope clear. Automatic straighten/crop-rotation helpers beyond the ordinary transform system are **POST_V1_CANDIDATE** unless promoted by an accepted scope decision.

# Clone — POST_V1_CANDIDATE

Source point + offset, aligned/non-aligned modes, sample current layer vs composite policy. These are deferred design notes, not V1 implementation authorization. When implemented, Clone reads an immutable source snapshot during stroke to avoid feedback artifacts unless explicitly supported.

# Heal — POST_V1_CANDIDATE

Deferred source/target + blending behavior; asynchronous preview may be used when the feature is formally promoted. Never market content-aware intelligence before reference behavior/tests exist.

# Target safety

Painting indicates active content vs mask clearly. Locked/non-raster target offers actionable switch/rasterize/create-layer workflow rather than silently painting nowhere.

# Tests

**V1 tests:** pressure/tilt, high-frequency samples, selection edges, masked brush, cancellation, huge sparse image, crop undo, 16-bit paths and CMYK PixelLayer behavior. **Post-V1 Clone/Heal tests**, including clone overlap, become required only when those features are formally promoted.

# Implementation contract — V1

## Scope status

**V1_REQUIRED:** rectangular/elliptical/lasso/freehand raster selection; New/Add/Subtract/Intersect modes; Select All/None/Invert/Grow/Shrink/Feather; Brush; Eraser; raster Gradient; crop; PixelLayer/mask target editing; pressure/tilt where hardware provides it; 8/16-bit paths.

**POST_V1_CANDIDATE:** Clone Stamp, Heal, content-aware fill/repair and advanced retouching. The Clone/Heal descriptions on this page are deferred design notes and must not be interpreted as V1 authorization.

## Raster selection ownership

The active pixel selection is **session/document-editing state, not canonical persisted artwork**. Represent it as `RasterSelectionState` (or equivalent) associated with the DocumentSession/view editing context. It may be expensive/tiled, but ordinary save/reopen does not depend on it.

Explicit commands such as **Save Selection as Mask/Channel** materialize canonical document data. Loading a saved mask/channel back into selection creates a new session selection snapshot. This prevents transient marching-ant state from polluting the native format.

## Selection transaction semantics

Selection tools use `begin/update/commit/cancel` and operate on a provisional coverage mask. New/Add/Subtract/Intersect combine coverage deterministically. Feather/grow/shrink can be either explicit commands after selection or tool parameters, but the same operation implementation must be reused.

Selection edges are continuous coverage values, not merely binary pixels. UI marching ants are derived visualization and never canonical mask data.

## Brush stroke contract

A brush gesture captures target ID/type, source revision, brush preset snapshot, color/paint mode, normalized pointer-device capabilities and selection/mask constraints before painting.

Pipeline follows 09.6: `raw input → normalize → stabilize/resample → dynamics → stamps → tile compositing → dirty regions → preview → atomic stroke commit`.

One stroke produces one undo transaction. A brush preset changed mid-stroke affects the next stroke unless an explicitly supported live parameter is part of that stroke snapshot.

## Target semantics

Valid brush targets are explicit writable PixelLayer or raster mask surfaces. Painting on a vector object, ImageObject, locked object or unsupported target must return an actionable state: create PixelLayer, rasterize explicitly, select mask/content target, unlock, or cancel. Never silently auto-rasterize source artwork.

Mask-edit mode must be visible through more than color alone and exposed in accessibility/inspection state.

## Eraser

Eraser delegates to the brush engine with typed erase semantics. V1 policy on a PixelLayer without usable alpha must be explicit per layer/document mode: enable/add alpha through a command where supported or reject with explanation; never silently paint an arbitrary background color.

## Raster gradient

Gradient rasterization uses the same semantic Gradient/ColorValue definitions where applicable, but commits pixels to the chosen PixelLayer/mask. It honors active selection coverage, bit depth and color profile. Preview may use GPU acceleration; final commit must match the reference semantics within tolerance.

## Crop

Distinguish:

- **Surface/Document Crop:** non-destructive change of visible/output Surface geometry when possible;
- **PixelLayer Crop:** explicit destructive pixel-storage command;
- **Export Crop:** output-only option and not a document mutation.

The Photo crop tool defaults to the Surface/document-level semantic crop unless the user selects the destructive PixelLayer command. Crop/rotate/straighten beyond baseline rotation must have explicit scope status before UI exposure.

## Clone/Heal deferred contract

`Clone` and `Heal` are `POST_V1_CANDIDATE`. Code agents must not add menu/tool entries, Actions or partially working buttons for them during V1 unless the scope registry is amended. When implemented later, they must use source snapshots, tiled undo, selection/mask/color semantics and the same target-safety rules.

## Performance and cancellation

Pointer interaction must not wait for full-image recomputation. Dirty tiles are scheduled incrementally; visible tiles outrank offscreen work. Cancellation before commit restores canonical pixels exactly. Memory pressure may evict clean tiles but never a transaction's required before-image before undo data is secured.

## Automation/plugins

MCP/Lua expose semantic selection operations, brush-stroke application for deterministic/batch use, crop commands and target queries. Ordinary scripting should not emulate raw high-frequency pointer events when a typed operation exists. Plugin tools may feed normalized tool input but cannot directly mutate tile storage.

## Required tests

Add selection-not-serialized test, Save/Load Selection materialization, coverage combine/feather boundaries, preset snapshot mid-stroke, wrong-target recovery, alpha-less eraser policy, 16-bit/profile gradient, crop scope distinction, memory-pressure stroke undo, cancellation at tile boundaries and verification that Clone/Heal are absent from V1 capability discovery.