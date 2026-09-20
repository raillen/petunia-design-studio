# 10.5 — Layers, Groups, Clips, Masks, Symbols, Styles, Assets & Resource Libraries

# Structural hierarchy

Layers panel presents the canonical object tree plus semantic badges. Group is structural container; Layer may be a named organizational/group semantic if kept distinct by ADR, but UI hierarchy must not invent separate truth.

# Reparenting

Drag indicates before/after/inside/clip/mask target distinctly. Invalid cycles rejected before preview commit. Reparent preserves world transform by compensating local transform unless user invokes alternative behavior.

# Clipping

ClipGroup/container defines clipping path(s) and content order. Clipping vs masking are distinct semantics and UI affordances.

# Masks

Vector/raster masks attach through explicit relationship and can be enabled, inverted, edited, released. Mask target editing mode is visually unmistakable.

# Symbols

Symbol definition is canonical reusable subtree/resource; instances reference definition plus allowed overrides. Editing definition updates instances. Detach makes independent subtree. Nested/cyclic definition rules explicit.

# Styles

Character/Paragraph/Object/Appearance styles have stable IDs, inheritance policy if supported, redefine/detach/apply semantics, and missing style recovery. Style changes propagate via dependency graph.

# Assets

Assets panel is a library/browser over resources or reusable templates, not an alternate document store. Dragging creates/copies/links according to asset type.

# Libraries

Built-in/document/user libraries separated. Cross-document insert performs resource dedup by fingerprint + stable remap, never assumes IDs globally unique.

# Thumbnails

Derived/cache only, async/cancelable, invalidated by relevant changes.

# Tests

Huge/deep trees, cycle attempts, world-transform-preserving reparent, clipping/mask release, symbol override invalidation, cross-document paste, missing linked resource, library unload.

# Implementation contract — V1

## Scope status

**V1_REQUIRED:** one canonical hierarchy; semantic Layers/Groups; reparent/reorder; clipping; vector/raster masks; Symbols; Character/Paragraph/Appearance styles; document assets/resources; built-in/document/user library browsing and safe cross-document insertion.

**POST_V1_CANDIDATE:** cloud-synchronized/shared remote libraries, collaborative library publishing and complex symbol override languages beyond the baseline contract.

## One-tree rule

Aubrieta must not maintain a hidden “Layers tree” separate from the canonical object hierarchy. A **Layer** is a semantic container role in that same hierarchy (for example a Group/container with `ContainerRole::Layer` or equivalent toolkit-neutral domain representation). The exact Rust representation may evolve, but there is only one ordering/parent truth.

The Layers panel may present filtered/flattened views, but drag/reparent always resolves back to canonical ObjectId relationships.

## Container semantics

Containers declare role and behavior explicitly: ordinary group, layer-like organizational container, clip group or other accepted structural role. Role changes are semantic commands and cannot silently change clipping/masking behavior.

- Groups inherit transform/opacity/isolation according to compositor rules.
- Layer-role containers are named organizational roots with visibility/lock semantics but do not create a second storage system.
- Clip/mask relationships are explicit typed edges, not inferred from row indentation alone.

## Reparent transaction

Before preview, validate target existence, cycle safety, capability/lock state and whether the requested drop mode is `before`, `after`, `inside`, `clip` or `mask`. Capture original parent/index/world transform. Commit applies all selected moves atomically and compensates local transforms to preserve visual world placement unless the selected command explicitly requests local-coordinate preservation.

If any element cannot be moved, default multi-object behavior is all-or-nothing unless a specifically documented partial command is used.

## Visibility, lock and isolation

Visibility and lock are document semantics where they affect authoring/output; panel collapse/selection highlight are session UI state. Lock prevents mutation through UI, MCP and plugins unless an explicit privileged command temporarily unlocks it. Hidden content must remain serialized and export behavior follows its visibility semantics.

## Masks and clips

A mask relationship has target, mask source, mode/type, enabled state, inversion and edit-target state outside the document where appropriate. Vector and raster masks share evaluation ordering but preserve native representation. `Release Mask/Clip` must define resulting object placement/order and be fully undoable.

## Symbol model

A `SymbolDefinitionId` owns a reusable canonical subtree/resource. `SymbolInstance` references the definition and a versioned override map limited to explicitly overridable PropertyIds. Definition edit propagates via dependency graph. Instance detach materializes the effective subtree atomically.

Symbol definition graphs must be cycle-free. Nested symbols are allowed only when cycle validation succeeds. Missing/corrupt definition produces a recoverable placeholder with preserved reference metadata.

## Styles

Style resources have stable IDs, type, version and optional parent only if style inheritance is explicitly supported for that style family. Local overrides are represented separately from base style identity. `Clear Overrides`, `Redefine Style`, `Detach Style` and `Delete Style` have deterministic migration/replacement behavior.

Deleting a referenced style cannot create dangling IDs; require replacement, detach/materialize or explicit fallback policy.

## Assets and libraries

An asset entry describes reusable content/resource metadata, thumbnail and insertion policy (`copy`, `link`, `instantiate symbol`, etc.). Libraries are catalogs, not alternate document stores. User-library content must be versioned and validated like other resource packs. Cross-document insertion builds a remap table before commit and deduplicates only when identity/content policy confirms equivalence.

## Large-tree behavior

Layers presentation must be virtualized and delta-driven. Tree expansion, row selection and scrolling are session/presentation state. Canonical object mutation must not depend on which rows are mounted in GPUI.

## Automation/plugins

Tree query/reparent, visibility/lock, symbol/style application and asset insertion are semantic Actions/Commands/Property schemas shared by UI, MCP and Lua. Plugins can contribute asset/library providers through capability contracts but cannot mutate panel tree state as document truth.

## Required tests

Add one-tree invariant, layer-role roundtrip, atomic multi-reparent failure, world-transform preservation across transformed parents, lock enforcement via UI/MCP/Lua, mask release ordering, symbol cycle rejection/detach, style delete/replacement, cross-document remap/dedup, 10k/100k presentation stress and module/library unload without document loss.