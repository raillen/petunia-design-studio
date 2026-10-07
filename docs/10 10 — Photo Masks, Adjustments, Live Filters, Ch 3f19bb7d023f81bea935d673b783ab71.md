# 10.10 — Photo Masks, Adjustments, Live Filters, Channels, Histogram & Analysis

# Adjustment architecture

AdjustmentNode has AdjustmentKind + typed parameters + mask/opacity/blend. It consumes underlying composited input according layer semantics and produces derived output. Canonical parameters are renderer-neutral.

# Baseline adjustments

Levels, Curves, Exposure, Brightness/Contrast, White Balance, HSL, Vibrance, Black & White, Channel Mixer, Gradient Map, Posterize/Threshold and Selective Color as color model implementation permits.

# Curves

Channel selector, Bezier/control points, histogram backdrop. Drag points stages preview; pointer up commits SetAdjustmentParams. Numeric input available. Points sorted/invariants enforced natively.

# Levels

Input black/white, gamma/mid, output black/white per channel. Histogram derived async. Clipping preview optional overlay.

# Live filters

Gaussian Blur, Motion Blur, Median/denoise candidate, High Pass, Unsharp Mask, Sharpen, Noise, Vignette, Distort/Perspective-compatible effects, procedural generator candidates. Each EffectKind declares ROI expansion, quality levels and GPU/CPU capability.

# Masks

Any adjustment/filter/group can receive pixel/vector mask where semantic. Mask thumbnail/target selection in Layers; Alt-click view mask solo; Shift-click disable convention if adopted and documented.

# Channels

Display composite and document channels/alpha/spot where applicable. Channel visibility is view state; selecting/editing a channel changes PixelTarget explicitly. Copy/paste/load channel as selection are Commands where canonical data changes.

# Histogram

Async analysis of selected scope: document composite, selected layer or selection. Bins depend bit depth/channel model. Panel shows clipping; stale computation discarded when revision changes.

# Scopes

Optional waveform/vectorscope/lab analysis post-V1 can use same AnalysisService interface.

# Nondestructive order

Layer/effect stack order is visible and reorderable according constraints. Cache invalidation is ROI-aware where possible.

# Filter dialog/panel

Live preview, split/compare optional, reset, numeric fields, quality indicator. Cancel discards staged parameter changes; Apply commits existing live effect or destructive apply according chosen action.

# Destructive filters

Explicit Apply Filter to Pixels runs immutable source snapshot -> output tiles -> atomic tile commit. UI distinguishes from Add Live Filter.

# Commands

AddAdjustment, SetAdjustmentParams, AddLiveFilter, SetFilterParams, ReorderEffect, AttachMask, InvertMask, ApplyFilterToPixels, ChannelToSelection, SelectionToChannel.

# MCP/plugins

photo.add_adjustment, photo.set_adjustment, photo.add_live_filter, analysis.histogram use same schemas/jobs. Plugin adjustments register typed effect descriptors and backend capability.

# Tests

Reference images at 8/16/float, per-channel correctness, masked adjustments, ROI seams, effect ordering, preview/final convergence, GPU/CPU tolerance and histogram stale cancellation.