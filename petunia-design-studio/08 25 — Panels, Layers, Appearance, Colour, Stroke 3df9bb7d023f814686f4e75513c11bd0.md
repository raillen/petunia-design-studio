# 08.25 — Panels, Layers, Appearance, Colour, Stroke, Transform, Assets & Symbols UX Contract

# Amendment 2026-09-21 — Panel micro-conformance

Petunia Slint panels must additionally pass the pixel/microinteraction contract in 08.35 and the interaction manifest/gauntlet in 08.36. No row action may appear only on hover without keyboard/context-menu equivalent where it is a required workflow. Mixed state, drag target, rename, visibility, lock, reset/revert, empty/error/loading and large-list performance are first-class states.

<aside>
🧩

**Panel rule:** a panel exists because users maintain a persistent mental model or browse a collection. Selection-specific one-off controls belong in Properties or context UI instead of multiplying panels.

</aside>

# Default panel priority

Layers → Properties → Colour/Swatches → Stroke/Appearance → Transform/Align → Typography when relevant → Assets/Symbols when browsing reusable content.

Workspace Profiles may rearrange these.

# Layers

Affinity Layers handles layers, groups, objects, artboards, clipping, masks, opacity, blending, adjustments and effects.[[1]](https://affinity.help/designer2/English.lproj/pages/Panels/layersPanel.html)

Petunia improvements:

- one canonical structural tree;
- deterministic drop zones with reorder/child/clip/mask previews;
- semantic row badges;
- virtualization from first implementation;
- type-to-filter without changing selection;
- reveal on canvas and reveal in tree;
- ancestor lock/visibility explanation;
- optional thumbnail suppression;
- multi-selection summary;
- full keyboard tree editing;
- accessible level, expanded, selected, locked and visibility semantics.

# Properties

Properties is the stable contextual inspector backed by typed PropertyDescriptors. Canonical section order is Transform → Appearance → Fill → Stroke → Blend/Opacity → Effects → object-specific → export/metadata.

Mixed selection shows common editable fields first and explains excluded properties.

# Appearance

Affinity permits multiple fills and strokes with ordering, visibility and per-entry blend state.[[2]](https://affinity.help/designer2/English.lproj/pages/Panels/appearancePanel.html)

Petunia adds stable AppearanceItem IDs, exact insertion previews, duplicate, disable-versus-delete distinction, optional naming, Expand/Bake preview and strong selected-state semantics rather than relying on a small dot.

# Colour

Affinity exposes HSL/RGB/CMYK/LAB input models, wheel/sliders/boxes and picker integration.[[3]](https://affinity.help/designer2/English.lproj/pages/Panels/clrPanel.html)

Petunia separates the **input model** from the **document semantic colour space** so users cannot confuse changing sliders with converting document/profile semantics. Show process/global/spot role, profile/source badge, recent/document/library swatches and optional gamut/proof warnings.

# Stroke

Canonical fields: style, width, alignment, cap, join, miter, dash pattern, phase, markers, scale-with-object and width profile. Affinity provides the reference interaction family.[[4]](https://affinity.help/designer2/English.lproj/pages/Panels/strokePanel.html)

Petunia adds a live mini-preview and direct jump to on-canvas Width Tool.

# Transform

Petunia standardizes X/Y/W/H, rotation, skew, 3×3 anchor, coordinate space, relative expressions, aspect policy, per-input unit override and selection-bounds versus intrinsic-geometry distinction.

# Assets

Affinity Assets provides category/subcategory browsing, search, list/grid and import/export.[[5]](https://affinity.help/designer2/English.lproj/pages/Panels/assetsPanel.html)

Petunia adds provenance, licensing metadata, document/local/library scope, tags, missing-dependency warning, deterministic insertion policy and cancellable thumbnail generation.

# Symbols

Petunia panel exposes definition versus instance, synchronization, overrides, edit definition, reveal instances, detach and cycle prevention. Affinity Symbols are the interaction prior art.[[6]](https://affinity.help/designer2/English.lproj/pages/SymbolsAssets/symbols.html)

# History

Affinity exposes labeled states and optional saved/branch-like future history.[[7]](https://affinity.help/designer2/English.lproj/pages/Panels/historyPanel.html)

Petunia V1 remains transactional linear undo/redo unless 09.3 explicitly promotes safe branch semantics.

# Constraints

Affinity Constraints uses anchoring/scaling relationships between parent and child objects.[[8]](https://affinity.help/designer2/English.lproj/pages/Panels/constraintsPanel.html)

Petunia adopts the mental model only where Functional Atlas scope supports it and provides textual/numeric equivalents for graphical controls.

# Panel state contract

Every panel documents no document, no selection, supported selection, mixed selection, loading, stale resource, error, read-only, capability-disabled, huge-data virtualization, keyboard traversal, search/filter, persistence, minimum useful size and responsive collapse.

# Panel-to-canvas bridge

Hover may highlight related canvas content but never changes document selection. Persistent selection changes only through the semantic Selection service.