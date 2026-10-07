# 19.02 — Properties Panel Specification

# Identity

PanelId [ptnd.panel.properties](http://ptnd.panel.properties).

# Source

Selection + active tool → PropertySchema aggregation. Never feature-specific mutation bypass.

# Sections

Stable grouping: Transform, Geometry, Appearance, Text, Layout, Raster, Effects, Metadata as applicable. Tool-only properties can appear pinned/top with clear scope.

# Values

Same(value), Mixed, Unavailable, Loading, Error. Mixed never rendered as fake default.

# Editing

PropertyBindingController opens preview transaction for scrub/slider; text/numeric field validates locally then authoritative core validation on commit.

# Multi-selection

Property applies to all compatible targets according schema; incompatible subset policy shown. No silent partial update unless property explicitly declares compatible-only behavior.

# Reset/inheritance

Reset to default, clear override, detach style/global reference are distinct actions.

# Search

Optional property search with synonyms; result retains section context.

# Accessibility/tests

Label/control association, units, mixed state, error explanation. Test rapid selection changes during edit, plugin property provider disappearance, multi-select and undo coalescing.