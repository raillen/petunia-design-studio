# 18.29 — Healing Tool

# Identity

ToolId ptnd.tool.heal.

# Source

Manual source anchor similar to Clone; optional auto-source mode only if algorithm/provider supports and UI labels it.

# Algorithm contract

IHealingEngine receives source/destination neighborhoods, mask, color context and parameters, returning staged pixel patch. It may adapt texture/illumination but result must be deterministic for fixed inputs/seed.

# Brush

Size/hardness/opacity plus alignment/source scope. Native halo region includes neighborhood required by algorithm.

# Preview

Low-latency preview can approximate while stroke active; final patch converges before commit under documented quality policy.

# Commands

CommitHealStroke.

# Errors

Insufficient source bounds, unsupported bit depth/color mode and cancelled jobs produce no partial mutation.

# Tests

Texture/tone changes, edges, transformed layers, high bit depth, deterministic output and undo.