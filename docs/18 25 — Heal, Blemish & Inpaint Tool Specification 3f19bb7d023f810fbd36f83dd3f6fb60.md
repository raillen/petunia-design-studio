# 18.25 — Heal, Blemish & Inpaint Tool Specifications

# Heal

Manual-source retouch preserving source texture while adapting local tone/color to destination. Uses same source grammar as Clone.

# Blemish

Spot/short-stroke retouch with automatic or adjustable source candidate. Candidate can be inspected before final commit if algorithm permits.

# Inpaint

User paints removal mask; engine/provider computes replacement using immutable image snapshot. Expensive computation returns JobId.

# State machines

Heal: NoSource -> SourceReady -> Stroke -> Commit.

Blemish: Hover -> MarkSpot -> Preview -> Commit.

Inpaint: MarkRegion -> Compute -> PreviewResult -> Commit/Cancel.

# Provider interfaces

IHealingEngine and IInpaintEngine. Built-in local implementation is baseline. Any network/AI provider must be explicit plugin/provider with network permission and disclosure.

# Context

Brush controls, source mode/scope, quality, output mode if live result layer is supported.

# Atomicity

No partial inpaint tiles become canonical before job completion. Cancel/provider crash leaves document unchanged.

# Privacy

Document pixels are never uploaded by built-in tool without explicit provider authorization path.

# Tests

Texture edges, transparency, masks, large jobs, provider crash, cancel, deterministic classical fixture and undo.