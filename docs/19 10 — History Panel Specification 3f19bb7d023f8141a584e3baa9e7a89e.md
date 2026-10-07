# 19.10 — History Panel Specification

# Model

Chronological committed transactions with current pointer and saved marker. Labels generated from Action metadata + compact parameters.

# Interaction

Click prior entry performs repeated undo/redo to target after confirming only if expensive/branch-impact policy requires. Arrow keyboard navigation does not mutate until activation if desired.

# States

Applied entries, redo entries, saved revision marker, pruned boundary and source badges optional for plugin/MCP diagnostics.

# Details

Expandable technical view can show affected object count, ActionId and source; artwork content not logged unnecessarily.

# Clear

Clear History explicit session action; not same as deleting document content.

# Performance

Virtualized model, large history labels lazy. Raster spill/compression status not spammed but diagnostics accessible.

# Tests

Branch after undo, saved marker crossing, pruned boundary, plugin action label and keyboard use.