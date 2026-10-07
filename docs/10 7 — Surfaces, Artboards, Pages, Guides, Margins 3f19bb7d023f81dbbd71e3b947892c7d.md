# 10.7 — Surfaces, Artboards, Pages, Guides, Margins, Columns, Bleed & Lightweight Layout

# Surface

Single canonical abstraction with role metadata: Artboard, Page, ExportRegion or general Surface. Role affects UI/defaults, not separate incompatible model.

# Surface properties

Size, position, orientation, background/display, bleed, margins, columns, grid association, export flag, template reference, page metadata.

# Creation

Surface Tool drag or preset. New Document can initialize one/multiple. Duplicate preserves content policy via explicit Duplicate Surface action.

# Content relation

Objects may belong to Surface root/container or global pasteboard according model. Moving Surface can optionally move contents; modifier/context toggle makes policy visible.

# Pages

Page order separate from arbitrary spatial artboard order when layout mode needs sequential pagination. Page panel can reorder. Page number metadata derived/field usable by text/data.

# Surface Templates

Reusable template/master-like definition applied to Surfaces. Instances inherit template content with controlled overrides. This avoids a separate Publisher architecture.

# Margins

Per-side values, linked toggle, visual overlay, snap targets. Margin guides are derived from Surface settings, not ordinary guide objects.

# Columns

Count + gutter or explicit columns. Overlay and text frame helpers can snap. Column changes are Surface property commands.

# Bleed

Per-side; exported when format/option supports. Overlay uses distinct non-content indication.

# Guides

Free horizontal/vertical guides can be document or Surface scoped. Numeric manage via Properties/guide UI; lock/hide/snap properties.

# Baseline grid

Document/Surface grid with start/spacing; text paragraph option aligns to grid. Renderer overlay derived.

# Rulers/origin

Document origin can differ from Surface local coordinates. Numeric UI clearly indicates coordinate space.

# GUI

Surface panel/page list, context toolbar dimensions/presets, canvas labels/handles, Layers filtering by Surface, status current Surface.

# Commands

CreateSurface, DeleteSurface, Resize/MoveSurface, ReorderPages, SetMargins, SetColumns, SetBleed, Add/Move/DeleteGuide, SetBaselineGrid, ApplySurfaceTemplate.

# Export

Surface defines common export target but exporter receives target list explicitly. Bleed/crop marks are output options, not renderer screenshot.

# Tests

Multi-surface transforms, moving with/without contents, page reorder, template inheritance, guides scopes, bleed export and huge canvas precision.

[10.7.1 — Surface/Page Model, Spatial Artboards vs Sequential Pages & Coordinate Systems](10%207%201%20%E2%80%94%20Surface%20Page%20Model,%20Spatial%20Artboards%20vs%20%203f19bb7d023f81aab8edd597e8536d68.md)

[10.7.2 — Margins, Columns, Baseline Grid, Guides & Layout Constraint Semantics](10%207%202%20%E2%80%94%20Margins,%20Columns,%20Baseline%20Grid,%20Guides%20&%203f19bb7d023f81af860fd5f6ae653931.md)

[10.7.3 — Surface Templates / Master-like Content, Overrides & Page Number Fields](10%207%203%20%E2%80%94%20Surface%20Templates%20Master-like%20Content,%20Ov%203f19bb7d023f816da955ef92a4144011.md)

[10.7.4 — Text Wrap, Object Exclusion Geometry & Layout Invalidation](10%207%204%20%E2%80%94%20Text%20Wrap,%20Object%20Exclusion%20Geometry%20&%20La%203f19bb7d023f816caa9ee222d79cfe8f.md)

[10.7.5 — Layout Preflight: Overset, Effective DPI, Bleed, Missing Resources & Page Integrity](10%207%205%20%E2%80%94%20Layout%20Preflight%20Overset,%20Effective%20DPI,%20%203f19bb7d023f816c8716f5b66a5bae85.md)