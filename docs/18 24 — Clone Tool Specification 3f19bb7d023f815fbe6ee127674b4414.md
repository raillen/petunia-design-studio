# 18.24 — Clone Tool Specification

# Identity

ToolId ptnd.tool.clone. Photo.

# Source setup

Alt/Option-click establishes source anchor in selected sampling scope. Source point and destination offset become tool state; not document state until stroke commit.

# Context

Brush preset/size/hardness/opacity/flow, Aligned toggle, source scope Current Layer / Current & Below / All Layers, blend mode where supported.

# Sampling snapshot

At stroke begin, engine captures coherent source snapshot policy to avoid recursive feedback. Current-layer live feedback, if ever offered, is a separate documented mode.

# State machine

NoSource -> SourceReady -> StrokeCapture -> Commit/Cancel.

# Overlay

Source crosshair, destination brush outline and optional offset connector.

# Aligned

Aligned preserves source-destination offset across strokes. Non-aligned restarts from original source point each stroke.

# Commands

CommitCloneStroke.

# Tests

Aligned/non-aligned, transforms, masks, source outside bounds, source/target color profiles, tile edges, undo and cancellation.