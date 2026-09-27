# 08.29 — Import, Place, Export, Preflight & Workflow Completion UX

<aside>
📤

**Workflow completion is part of UX.** Export, place/import, missing-resource recovery and print preparation must feel like continuations of editing rather than unrelated utility dialogs.

</aside>

# Affinity reference

Affinity's Export Persona allows designated slices/areas and batch-oriented export configuration.[[1]](https://affinity.help/designer2/English.lproj/pages/ExportPersona/exportPersona.html)

The current 2026 Affinity training still teaches complete vector/pixel/layout workflows ending in final export.[[2]](https://www.canva.com/design-school/resources/full-affinity-workflow/)

# Aubrieta export entry points

- Quick Export from selection/Surface;
- Export dialog for one configured output;
- Batch Export / export presets;
- reusable export metadata on Surface/object where 10.x supports it.

All resolve to the same Export contracts.

# Export dialog structure

1. target/area;
2. format;
3. size/scale;
4. colour/profile/transparency;
5. format-specific options;
6. fidelity/preflight warnings;
7. destination and overwrite policy;
8. estimated result summary;
9. Export.

Advanced options are progressive disclosure.

# Preflight

Before writing, surface:

- unsupported feature degradation;
- missing font/link/resource;
- colour/profile conversion;
- transparency/rasterization consequence;
- invalid output path/permissions;
- oversized output/resource estimate.

Warnings distinguish blocker, fidelity warning and informational conversion.

# Export presets

Presets are named, versioned configurations. User presets do not duplicate exporter business logic and remain portable through typed schema.

# Batch export

Jobs are individually observable and cancelable where safe. One failure does not necessarily cancel unrelated outputs. Summary reports success, warning and failure with paths and diagnostics.

# Place/import

Placing content uses a staging flow when interpretation is needed. Simple images can follow click-to-place or drag-to-size interaction like Affinity.[[3]](https://affinity.help/designer2/English.lproj/pages/Tools/tools_placeimage.html)

Embed/Link policy and resource status are explicit.

# Recovery

Missing linked resources offer Locate, Replace, Search configured roots and Keep Missing Placeholder. Recovery never silently substitutes a different file.

# Print-oriented flow

Design Persona exposes bleed/profile/preflight at the point users need them. Aubrieta does not bury production-critical state solely in Document Setup.

# Accessibility

Dialog path is keyboard-completable, labels include units, warnings have semantic severity, and progress is announced without flooding assistive technology.

# Tests

Quick export, multi-Surface export, overwrite, read-only destination, disk full, cancellation, colour-profile warning, missing font, corrupt linked asset, huge output, plugin exporter failure and save/reopen preset stability.