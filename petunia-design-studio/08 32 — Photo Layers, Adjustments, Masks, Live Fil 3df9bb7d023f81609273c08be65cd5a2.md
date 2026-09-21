# 08.32 — Photo Layers, Adjustments, Masks, Live Filters, Channels & Analysis UX

<aside>
🎚️

**Photo panels make non-destructive state legible.** Layers, adjustments, masks, live filters, channels, histogram and brushes are coordinated views over the same document semantics, not independent mini-applications.

</aside>

# Layers as the structural truth

Affinity Photo's Layers model contains pixel/image layers, masks, adjustment layers, live filters, clipping and groups.[[1]](https://affinity.help/photo2/English.lproj/pages/Panels/layersPanel.html)

Aubrieta keeps the same single structural-tree principle used by Design. Raster-specific badges include adjustment type, live filter, pixel mask, channel-derived mask, linked image and rasterized state.

# Adjustment layers

Affinity applies corrections/enhancements as adjustment layers; they may affect layers below or be nested to target one layer/group.[[2]](https://affinity.help/photo2/English.lproj/pages/Layers/adjustmentLayers.html)

Aubrieta:

- defaults to nondestructive adjustment objects;
- shows target scope visually in Layers;
- supports inline enable/bypass;
- keeps adjustment parameters in one standard editor surface;
- allows selection-created masks without hiding the resulting mask object.

# Adjustment browser

Affinity's Adjustment panel exposes preset thumbnails and adjustment types such as Curves, Levels, HSL, Exposure, White Balance and more.[[3]](https://affinity.help/photo2/English.lproj/pages/Adjustments/adjustment_applying.html)

Aubrieta organizes by intent plus searchable canonical name:

- Tonal;
- Colour;
- Detail;
- Output/Proof;
- Utility/Other.

Search aliases include familiar industry terminology without changing canonical Action IDs.

# Live filters

Affinity supports Live Filters as nondestructive layer-based filters.[[4]](https://affinity.help/photo2/English.lproj/pages/Layers/livefilters.html)

Aubrieta requires every filter to declare:

- live/nondestructive availability;
- destructive/bake alternative;
- maskability;
- colour-space/bit-depth constraints;
- preview cost;
- GPU/CPU fallback;
- serialization parameters.

# Before / after

Adjustment and filter editing provides a standardized bypass/compare affordance. It must compare against the transaction baseline without creating hidden history states.

# Curves / Levels

High-frequency tonal editors use an expandable graph surface with channel selector, histogram backdrop, numeric points and accessible point list. Keyboard users can add/select/nudge points through non-canvas controls.

# Masks

Mask objects display type and target scope. Painting a mask uses the shared brush grammar but status copy says **Mask**, not pixels. Invert, disable, delete, refine and convert/materialize operations are explicit.

# Channels

Affinity exposes composite and per-channel access and can derive selections/masks from channel information.[[5]](https://affinity.help/photo2/English.lproj/pages/Channels/usingChannels.html)

Aubrieta Channel panel:

- clearly distinguishes document colour channels, alpha and saved/spare channels;
- never equates hiding a display channel with deleting information;
- exposes Load as Selection / Save Selection / Create Mask as named actions;
- validates bit depth/colour-model availability through capability predicates.

# Histogram

Affinity provides Histogram as a persistent analysis panel.[[6]](https://affinity.help/photo2/English.lproj/pages/Panels/histogramPanel.html)

Aubrieta supports:

- composite/channel views;
- source scope;
- cached/asynchronous computation;
- stale/update indicator during heavy edits;
- clipping indicators where useful;
- no canvas-input stalls while histogram recomputes.

# Brushes panel

Affinity's Brushes panel manages brush categories and presets.[[7]](https://affinity.help/photo2/English.lproj/pages/Panels/brushesPanel.html)

Aubrieta adds provenance, search/tags, favorite/recent, target-tool compatibility, pressure/dynamics summary, editable-copy versus readonly-library state and cancellable thumbnail generation.

# Navigator / Info / Scope

Navigation and analysis panels are optional persistent tools. They must not duplicate document state. Navigator viewport frame is synchronized with the canonical Viewport state; Info samples are derived; scope analysis jobs are cancellable.

# Adjustment placement

When creating an adjustment from a selection, the UI previews whether it will be:

- above selected layer;
- clipped/nested to selection target;
- at top of stack;
- accompanied by a mask.

The resulting structural location is highlighted after commit.

# Performance policy

Histogram/scopes/thumbnails/preview filters run as derived jobs with priorities lower than pointer/brush interaction. Panels may show stale-but-marked data rather than block editing.

# Error/recovery

A disabled adjustment/filter exposes reason: unsupported bit depth, missing backend, invalid target or unavailable plugin. Failed live-filter computation preserves parameters and document state and offers retry/bypass.

# Accessibility

Graphs expose numeric/tabular alternatives. Layer tree semantics include type, target, enabled/bypassed, mask relationship and error state. Preset thumbnails have useful names, never colour-only identification.

# Canonical workflows

Photo correction: select layer → add adjustment → tune → mask → compare → save.

Retouch: select safe writable target → Clone/Heal/Inpaint → inspect result → undo/redo.

Mixed: Design composition → select image → add raster adjustment → switch back to vector tools without changing object identity.

# Tests

Nested adjustment scope, mask creation from selection, live-filter bypass, channel-derived mask, large histogram, 16-bit source, missing backend, plugin filter failure, rapid parameter scrub, cancel, save/reopen and headless/MCP parity for parameter changes.