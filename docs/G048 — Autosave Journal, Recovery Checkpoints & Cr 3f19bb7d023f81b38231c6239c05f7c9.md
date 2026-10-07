# G048 — Autosave Journal, Recovery Checkpoints & Crash-Safe Restoration

# Goal

Deliver production recovery independent from explicit PTND save.

# Depends

G005, G009–G012, raster history.

# Primary

editor-engineer + security/reliability review.

# Deliverables

journal/checkpoint format; trigger policy; dirty snapshot/delta persistence; startup discovery; Recovery UI; discard/cleanup; fault injection.

# Acceptance

Kill process at representative edits and recover latest valid state without overwriting original; journal failure does not corrupt doc.

# Tests

Truncation, disk full, stale source, raster deltas, migration and clean shutdown.