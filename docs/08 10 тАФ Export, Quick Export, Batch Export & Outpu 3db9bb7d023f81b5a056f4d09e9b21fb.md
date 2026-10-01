# 08.10 — Export, Quick Export, Batch Export & Output Window

# Export architecture

Export is a dedicated subsystem, not a Persona. It supports fast common output and deep format-specific control without cluttering the Design/Photo shell.

# Quick Export

Available from toolbar/menu/context for current selection/Surface using saved preset. If no preset exists, opens full Export dialog.

Quick Export commands may include:

- Export Selection as PNG;
- Export Current Surface as PDF;
- Export All Marked Surfaces;
- Copy as SVG/PNG where practical.

# Full Export window/dialog

Preferred as resizable modal sheet or dedicated owned window if preview/settings complexity benefits from space.

Three-column conceptual layout:

1. left target list — **V1 Required:** Document, Surfaces and Selected Objects; Slices/asset-export constructs are **Post-V1 Candidate** unless promoted;
2. center preview with page navigation/zoom;
3. right format/settings inspector.

Bottom action row: destination summary, estimated output, `Cancel`, `Export`.

# Format selector

Top of settings inspector. Categories:

- PDF;
- SVG;
- PNG;
- JPEG;
- TIFF;
- WebP if supported;
- plugin-contributed formats through the same Exporter registry/capability contract when the Plugin SDK exporter contribution is available; format plugins do not receive a parallel export path.

Changing format preserves only compatible settings and clearly indicates resets.

# Common export settings

- dimensions/scale;
- DPI/PPI where rasterization relevant;
- color model/profile;
- embed/convert profile;
- background transparency;
- resampling method;
- selection/surface bounds;
- bleed inclusion;
- metadata/privacy policy;
- overwrite naming policy.

# PDF settings

At minimum:

- all/current/range of Surfaces/pages;
- preserve vector/text where possible;
- font embedding/subsetting policy as exporter supports;
- image downsampling;
- color conversion/profile;
- bleed/crop marks only if supported;
- compatibility/profile preset;
- hyperlinks/bookmarks are **Post-V1 Candidate** unless explicitly required by the PDF functional contract.

Native PDF/X is explicitly **Post-V1 Candidate / deferred by product decision**. V1 provides professional color-managed PDF plus preflight and may hand off to external PDF/X validation/conversion; a backend gaining support does not silently promote PDF/X without an ADR.

# SVG settings

- precision;
- text as text/paths;
- style attributes strategy;
- responsive/viewBox behavior;
- embed/link raster images;
- minify optional;
- preserve IDs/names optional.

# Raster settings

PNG: bit depth/alpha/profile.

JPEG: quality, chroma behavior if exposed, background flatten color, metadata.

TIFF: bit depth, compression, CMYK/profile where supported.

# Preview

Preview shows target bounds, transparency checkerboard where relevant and export-resolution warning. Expensive previews render asynchronously with generation token so stale jobs do not overwrite newer settings.

# Estimated size

Only show when estimate meaningful. Label as approximate.

# Presets

Built-in and user presets. Operations: Save Preset, Update, Duplicate, Rename, Delete user preset, Reset built-in. Presets are format-scoped with shared common-field inheritance if architecture supports it.

# Batch export

Batch window/table lists targets with:

- enabled checkbox;
- source Surface/selection;
- output format/preset;
- scale variants;
- filename pattern;
- destination.

Batch actions: Add Variant, Duplicate Variant, Remove, Export All. Progress list allows cancel remaining, retry failed and reveal completed destination.

# Filename templates

Tokens such as `{document}`, `{surface}`, `{index}`, `{scale}`, `{record}`. UI provides token insertion menu and live filename preview. Invalid filesystem characters sanitized with visible explanation.

# Overwrite conflicts

Before output, choose policy:

- Ask;
- Replace;
- Skip;
- Rename with suffix.

For batch, `Apply to All` available.

# Export progress

Nonblocking for long tasks. Background-task popover/status indicator shows current item, percentage if known, cancel and errors. Successful export can toast with `Show in Folder`.

# Export error handling

Errors identify specific target and format setting; do not show generic `Export failed`. Batch continues when safe and summarizes failures.

### Implementation cross-reference — 2026-10-01

Scope: **Milestone Required (MVP)**. The desktop file-workflow adapter resolves
configuration, stable save/close targets, normalized export destinations and PNG
DPI. Native-system and assistive-technology acceptance remain open. See
[ADR-007](../docs/developers/adr/ADR-007-desktop-file-workflows.md) and
[implementation/evidence mapping](../docs/developers/uiux-file-workflows.md).
This note does not promote ICC/CMYK or basic PDF to production-ready capability.
