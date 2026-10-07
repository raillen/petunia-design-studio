# 18.27 — Selection Brush Tool Specification

# Purpose

Paint soft/hard selection mask using brush-like cursor, optionally edge-aware.

# Target

PixelSelection only. Does not paint layer pixels.

# Modes

Add/Subtract primary; New initializes empty selection on first stroke if configured. Intersect is not natural brush mode and may remain command/context operation.

# Input

Brush size/hardness/pressure optional. Edge-aware mode samples target/composite image and modifies brush coverage based on local edge model.

# Overlay

Selection overlay/marching ants update incrementally. Cursor shows add/subtract sign/color-independent icon.

# History

Each stroke can be selection-history entry; consecutive strokes need not pollute document history if selection history separated. Conversion to mask is canonical.

# Tests

Soft edge, add/subtract, edge-aware at tile boundary, pressure and selection-to-mask.