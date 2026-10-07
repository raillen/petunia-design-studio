# 18.26 — Dodge, Burn, Smudge, Blur & Sharpen Brush Specifications

# Shared engine

All use BrushStrokeSession and explicit PixelTarget.

# Dodge/Burn

Parameters: Range Shadows/Midtones/Highlights, exposure/strength, protect tones. Operation uses documented luminance/tone model, not arbitrary encoded-channel multiplication.

# Smudge

Maintains pickup/mix state along stroke. Parameters strength, flow, wetness/pickup if supported, sample scope. Tile-neighbor reads must be seam-free.

# Blur

Local blur kernel with radius/strength. Edge policy and alpha handling match effect reference implementation.

# Sharpen

Local sharpen/unsharp-like operator with strength/radius as defined. Prevent unstable repeated overshoot through bounded arithmetic/pixel format conversion.

# Nondestructive alternative

Direct pixel brush is destructive but undoable. Future live retouch layer is separate feature and not implied.

# Commands

CommitRetouchStroke with operator kind and settings snapshot.

# Tests

Tone ranges, alpha, high bit depth, tile seams, repeated strokes, GPU/CPU parity and undo.