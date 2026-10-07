# 18.24 — Eraser Tool Specification

# Identity

ToolId ptnd.tool.eraser; shortcut E.

# Engine

Reuses brush sampling/dynamics pipeline but operator is erase/alpha reduction rather than painting a background color.

# Target semantics

Raster pixels: reduce alpha/coverage according brush operator.

Pixel mask: erase mask value toward 0.

Channels: only if explicit supported target.

Vector/text: disabled unless explicit rasterize.

# Context

same core brush fields plus erase mode/opacity/flow and target indicator.

# Alpha/color

Erasing premultiplied compositor data must preserve canonical storage alpha/color policy and prevent dark fringes. CPU reference tests transparent colored pixels.

# History

One stroke transaction with tile COW.

# Accessibility/tests

Parameters keyboard accessible. Test semi-transparent pixels, mask erasing, selection restriction, tile edges and undo.