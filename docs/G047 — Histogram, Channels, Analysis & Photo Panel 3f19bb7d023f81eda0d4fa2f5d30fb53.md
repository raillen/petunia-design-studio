# G047 — Histogram, Channels, Analysis & Photo Panel Completion

# Goal

Complete Photo inspection/analysis UI.

# Depends

G039–G046, 19.11–19.12.

# Primary

editor-engineer + ui-component-engineer.

# Deliverables

async histogram service/panel; Channels model/PixelTarget; Info sampling; Adjustments catalog; mask target UI; analysis job coalescing.

# Acceptance

Histogram/channels react correctly to revision/selection without blocking painting; target is always explicit.

# Tests

RGB/CMYK, selection scope, stale jobs, keyboard/accessibility and high-frequency sampling.