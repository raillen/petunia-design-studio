# 03.5 — Retouch: Clone, Heal, Inpaint, Blemish, Dodge/Burn, Smudge, Blur & Sharpen

# Clone

User selects source with modifier; source crosshair and destination cursor both visible. Aligned/non-aligned sampling and source scope explicit.

# Heal

Uses source texture with destination tone/color adaptation. Source can be manual or tool mode-specific auto selection.

# Inpaint

User paints/removes region; engine computes replacement asynchronously. Algorithm can be classical built-in or permissioned plugin/provider. Network AI is never implied.

# Blemish

Optimized spot/short-stroke healing. Source suggestion visible/adjustable when meaningful.

# Dodge/Burn

Range selector Shadows/Midtones/Highlights, exposure and protect-tones option. Nondestructive recommended workflow may use dedicated live paint/effect representation; direct pixel mode remains explicit.

# Smudge

Carries pixel sample state along stroke. Native engine controls pickup/mix and tile neighborhood.

# Blur/Sharpen

Brush-local operators with strength. Output remains color/bit-depth correct.

# Source sampling

Current Layer, Current & Below, All Layers options use immutable snapshot at stroke start or documented live policy to avoid temporal inconsistency.

# Cancellation

Brush strokes cancel before commit. Inpaint long job can cancel and leaves no partial tiles.

# Tests

Source alignment, transformed layers, masks, high bit depth, edge tiles, color-managed layers, undo and job cancellation.