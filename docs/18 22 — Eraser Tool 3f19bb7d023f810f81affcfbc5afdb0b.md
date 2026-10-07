# 18.22 — Eraser Tool

# Identity

ToolId ptnd.tool.eraser. Shortcut E.

# Engine

Uses the same BrushEngine pipeline as Pixel Brush with erase operator.

# Target semantics

Raster pixels reduce alpha/coverage while preserving premultiplied correctness. Pixel mask paints lower coverage. Channel erase is allowed only when channel semantics define a neutral/default value.

# Modes

Erase to transparency baseline. Background-color erase, if offered, is a distinct paint action.

# Context

Preset, size, hardness, opacity, flow, stabilizer, target indicator, erase mode.

# Protected alpha

Lock-alpha state changes/blocks erase visibly and explains why.

# Commands

CommitEraserStroke through raster delta history strategy.

# Tests

Premultiplied fringe, opaque/transparent pixels, mask erase, alpha lock, high bit depth and tile seams.