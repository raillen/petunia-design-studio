# 18.20 — Eraser Tool Specification

# Identity

ToolId ptnd.tool.eraser. Persona Photo. Default shortcut E.

# Engine

Uses the same BrushStrokeSession with Erase operator semantics.

# Target semantics

RasterLayer: modifies alpha/coverage according canonical premultiplied model while preventing color fringes.

PixelMask: writes mask values according erase direction.

Other targets: disabled with reason.

# Context

Brush preset, size, hardness, opacity, flow, spacing, stabilizer and erase mode if multiple modes are explicitly supported.

# Cursor/overlay

Same geometry as Pixel Brush plus clear eraser state indicator and current target badge.

# Commands

CommitBrushStroke with Erase operator metadata; one stroke equals one history transaction.

# Color/alpha

Zero-alpha color handling follows raster spec; eraser must not create undefined/unbounded unpremultiply values.

# Tests

Alpha fringes, full/partial erase, masks, 8/16/float, tile seams, undo and export.