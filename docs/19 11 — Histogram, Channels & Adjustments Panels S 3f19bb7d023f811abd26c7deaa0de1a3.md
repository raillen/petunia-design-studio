# 19.11 — Histogram, Channels & Adjustments Panels Specification

# Histogram

PanelId ptnd.panel.histogram. Scope Document Composite, Selected Layer or Selection; channel/model selector; clipping indicators. Native analysis async and revision-aware.

# Channels

PanelId ptnd.panel.channels. Composite/components/alpha/spot entries. Visibility is view state; active editable channel is explicit PixelTarget.

# Actions

Channel to Selection, Selection to Channel, duplicate/create alpha where supported, delete custom channel, solo visibility.

# Adjustments

PanelId ptnd.panel.adjustments. Searchable catalog plus selected adjustment editor. Add creates AdjustmentNode; params generated from schema or custom editor.

# Curves

Graph supports point add/move/delete with numeric point table for accessibility. Histogram backdrop derived.

# Stale data

Histogram/analysis job result discarded if source revision/scope changed.

# Tests

8/16/float, RGB/CMYK, selection scope, mask target, curves keyboard, async cancellation and panel switching.