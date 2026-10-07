# 13.4 — Functional Atlas 10.1–10.13 Coverage Audit

# One-to-one

Legacy 10.1–10.13 subjects retain the same numbering and scope:

10.1 selection/transform/snapping;

10.2 paths/nodes;

10.3 shapes/boolean;

10.4 appearance;

10.5 layers/resources;

10.6 typography;

10.7 Surfaces/layout;

10.8 perspective/warp;

10.9 Photo tools;

10.10 Photo adjustments/masks;

10.11 Data Merge;

10.12 Design↔Photo;

10.13 functional gauntlet.

# Expansion

New pages explicitly add Python ToolController vs C++ engine boundary, Commands, GUI interaction, MCP/plugin exposure, accessibility, performance and edge-case tests.

# Functional completeness rule

A row can move from Spec Complete to Implemented only if its tool/function contract has concrete Action/Command and executable tests. UI mock alone is not functional implementation.