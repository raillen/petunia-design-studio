# 10.7 — Surfaces, Artboards, Pages, Guides, Margins, Columns, Bleed & Lightweight Layout

# Surface semantics

One canonical `Surface` can behave as artboard/page/export region through metadata/context. Design UI may call them Artboards or Pages based on workflow, but serialization uses one model.

# Surface tool

Create by drag or preset; numeric width/height/unit/orientation; duplicate; reorder; resize to selection/content; move surface with/without content policy.

# Coordinates

Define document/global vs Surface-local coordinate spaces. Objects belong to a Surface root or explicit global/pasteboard context. Moving between Surfaces preserves world/local transforms according to command.

# Margins/columns

Nonprinting layout metadata with per-side margin, column count/gutter or explicit column guides. Can derive guides but remain semantically distinct.

# Guides

Horizontal and vertical guides are **V1_REQUIRED** with lock/visibility/scope. Angled guides are **POST_V1_CANDIDATE** unless promoted by an accepted scope decision. Drag from ruler, numeric manager, delete by drag off/rule. Guide color may be user preference/token but document-specific guide custom color only if scope warrants.

# Baseline grid

Origin, spacing, scope and visibility. Text snapping/layout integration explicit; it is not a visual-only grid if text alignment uses it.

# Bleed

Per-side or linked values; display overlay; export adapters query Surface bleed semantics. Bleed never changes object bounds.

# Reusable/master-like elements

**Resolved product direction:** Aubrieta will not introduce a Publisher-style Master Pages subsystem. Reusable page/surface content is built from the existing reusable-object architecture: Symbols plus an optional lightweight `SurfaceTemplate` reference that points to reusable layer/symbol content.

A Surface may reference zero or more template/reusable roots for repeating backgrounds, headers, footers, guides or branded elements. Instances remain references, not duplicated hidden page trees. Initial overrides should stay deliberately small — visibility, transform where appropriate, text/data bindings and explicitly declared overridable properties. If a **Post-V1 publishing requirement** proves this lightweight model insufficient, extend the same reusable-content mechanism rather than creating a second document hierarchy.

# Multi-surface export

Order, naming, export inclusion flags and slicing rules deterministic.

# Tests

Mixed units, negative pasteboard coordinates, move/duplicate with content, guide snapping, page reorder, bleed export, large 1000-Surface docs, data-merge generated surfaces.

# Implementation contract — V1

## Scope status

**V1_REQUIRED:** unified Surface model, artboard/page naming by context, create/duplicate/delete/reorder/resize, local/global coordinates, margins, columns, baseline grid, horizontal/vertical guides, bleed, export inclusion/order and lightweight `SurfaceTemplate` references for reusable repeating content.

**POST_V1_CANDIDATE:** angled guides and publishing features that imply long-document/master-page architecture beyond the lightweight model.

## Surface identity and ordering

Every Surface has stable `SurfaceId`, name/metadata, geometry, ordering key/index, export-enabled state and optional template references. Reordering changes document Surface order but never object identity. Deleting a Surface requires explicit policy for its root objects: delete with Surface, move to pasteboard/another Surface or cancel; no orphaning by accident.

## Surface-local vs global coordinates

Objects attached to a Surface have local transforms relative to the Surface root; the document/pasteboard has global coordinates. Moving an object between Surfaces is one command that computes the destination local transform needed to preserve world appearance by default. “Move Surface with contents” and “Move Surface only” are distinct semantic actions.

Properties/MCP must expose which coordinate space a value uses.

## Surface resizing

Resize operations declare anchor/reference point and whether content is unaffected, proportionally transformed or constrained by an explicit layout command. Default artboard/page resize changes Surface geometry only; ordinary contained objects keep their local transforms unless the user chooses another policy.

## Layout metadata

Margins, columns, bleed and baseline grid are semantic Surface layout metadata and therefore serializable/automatable. Derived visual guides may be generated from them, but deleting a derived visual representation must not destroy the semantic layout setting unless the user invokes the corresponding layout command.

## Guides

V1 canonical guide types are horizontal/vertical with coordinate, Surface/global scope, lock and visibility. Guide display color is workspace/user presentation unless a documented document-specific guide-color property is introduced. Angled guides are `POST_V1_CANDIDATE` and must not appear as partially supported V1 controls.

Guide snapping uses the shared Snap service; guide lock affects editing, not snapping visibility unless configured.

## SurfaceTemplate baseline

`SurfaceTemplate` is a lightweight reusable composition reference, not a parallel page/master tree. A Surface can reference versioned reusable roots/symbol definitions for background/header/footer/brand elements and explicitly declared overrides. Minimal V1 override classes: visibility, approved text/data-bound properties and transforms only where the template definition marks them overridable.

Template references must be cycle-free. If a referenced template becomes unavailable, preserve the reference/override payload and show a recoverable missing-provider state. Detaching materializes the effective content atomically.

## Page/artboard terminology

“Artboard” and “Page” are localized presentation terms chosen by workspace/workflow; they are not different persisted object classes. Plugins/MCP use `Surface` semantics and can query presentation role metadata.

## Multi-surface performance

Surface lists/thumbnails are virtualized. Offscreen Surface rendering, thumbnails and preflight run as cancelable low-priority jobs. 1000-Surface documents must not instantiate one heavy UI/render context per Surface.

## Export/preflight

Export order is canonical Surface order unless a preset specifies another deterministic order. Bleed, inclusion flags, names, dimensions/color context and missing/overset/resource diagnostics are queryable without initializing the GUI.

## Required tests

Add Surface deletion policies, move-between-Surface transform preservation, Surface-only vs Surface-with-content move, resize anchor semantics, layout metadata↔derived guide separation, missing/cyclic SurfaceTemplate, detach template, 1000-Surface memory/virtualization, Data Merge generation and headless/MCP parity.