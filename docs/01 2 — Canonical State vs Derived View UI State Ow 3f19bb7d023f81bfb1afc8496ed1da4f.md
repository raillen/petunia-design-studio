# 01.2 — Canonical State vs Derived/View/UI State Ownership Matrix

# Canonical

Document hierarchy, object properties, text stories, raster tiles, masks, effects, resources, styles, symbols, bindings and semantic color data.

# Derived

Bounds, spatial index, tessellation, boolean result, shaped glyphs/layout, thumbnails, histograms, mips, effect intermediates, render scene and export staging.

# View/session

Selection, sub-selection, current Surface, viewport, active Persona/tool, proof mode, snap overlay, temporary tool preview and pixel selection if product policy keeps it session-scoped.

# UI/workspace

Dock tree, panel expansion/filter, window geometry, theme/density, command-palette query.

# History/recovery

History payload and recovery journal are session/application persistence, not canonical PTND truth.

# Ownership

C++ owns canonical/derived engine state. Python/Qt own view/presentation state but use typed IDs/snapshots. No duplicated canonical truth in Python models.

# Synchronization

ChangeSet carries semantic invalidation from canonical commit. UI models query/project new snapshot. Renderer rebuilds affected derived state.

# Test

Delete all derived caches and reopen/render: canonical result must remain identical.