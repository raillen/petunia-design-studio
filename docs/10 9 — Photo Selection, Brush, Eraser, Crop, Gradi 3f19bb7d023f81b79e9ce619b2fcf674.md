# 10.9 — Photo Selection, Brush, Eraser, Crop, Gradient, Clone, Heal & Retouch Tools

# Pixel target

Every pixel-modifying tool resolves a PixelTarget: RasterLayer pixels, PixelMask or editable channel. UI displays target. Locked/non-pixel target disables action with reason.

# Brush stroke

Begin captures preset snapshot, target revision and input settings. Samples contain document position, timestamp, pressure, tilt, rotation and buttons. Native engine resamples in stroke space, stabilizes, computes dynamics and dabs.

# Brush preset

Tip, texture, spacing, scatter, size/opacity/flow dynamics, rotation, blend, smoothing/stabilizer. Randomness uses deterministic stroke seed stored with history/evidence when reproducibility needs it.

# Eraser

Uses same brush pipeline with erase semantic. Mask erasing modifies mask values; layer erasing affects alpha according pixel format.

# Marquee/Lasso/Selection Brush

All output PixelSelection staging mask. Combine mode New/Add/Subtract/Intersect is explicit and shared. Feather can be applied live/non-destructively as selection property until command resolves.

# Magnetic selection

Edge analysis derived from raster snapshot; anchors user-visible. Cancel discards. Native job coalesces pointer updates.

# Refine

Input selection -> edge matte analysis -> user brush foreground/background/unknown hints -> parameters -> output selection/mask. Workbench preview quality may be lower; final commits full.

# Flood Fill

Connectivity and tolerance computed in chosen sample source. Sample Merged requires composited snapshot at known revision. Fill to target uses selection mask and color/pattern source.

# Raster Gradient

Generate gradient pixel operation within target/selection or insert live generator/effect according chosen mode. Same GradientDefinition schema where practical.

# Crop

Crop rectangle can alter Surface/document view non-destructively; trim pixels is a distinct destructive action. Straighten composes rotation + crop. UI must label which result type is selected.

# Clone

Source anchor stored per stroke operation state, not permanently unless tool settings. Sampling source options Current, Current & Below, All. Aligned toggle determines source offset across strokes.

# Heal

Samples source then blends texture/illumination. Algorithm behind IHealingEngine so implementations can evolve without tool contract change.

# Inpaint

Mask region + content snapshot -> cancellable native job -> staged raster patch. Commit only on complete. Plugin/AI implementation may contribute engine but must obey privacy/network permissions.

# Dodge/Burn

Operator adjusts tonal range. Direct pixel mode and nondestructive filter/paint-layer strategy are separate commands.

# Smudge/Blur/Sharpen

Native brush operators; smudge has pickup/carry state within stroke. CPU/GPU backend chosen by engine.

# Commands

CommitBrushStroke, SetPixelSelection, ApplyFloodFill, ApplyRasterGradient, SetCrop, TrimToCrop, CommitCloneStroke, CommitHealStroke, ApplyInpaintResult, CommitRetouchStroke.

# Undo

Touched tiles use copy-on-write/delta policy. One stroke one history item. Huge operations can store compressed snapshots subject memory budget.

# Tests

Pressure/tilt fixtures, deterministic seed, tile-boundary dabs, masks, alpha, selection combine, clone alignment, cancel inpaint, crop no-loss, 16-bit/float paths and color-managed sampling.