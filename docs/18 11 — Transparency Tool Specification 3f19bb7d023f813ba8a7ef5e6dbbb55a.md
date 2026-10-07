# 18.11 — Transparency Tool Specification

# Identity

ToolId ptnd.tool.transparency. Design.

# Purpose

Edit scalar opacity gradient or mask independently from fill color.

# Canonical model

Transparency appearance node stores gradient geometry, scalar opacity stops and composition target. It is evaluated at a defined appearance/compositor stage.

# State machine

Idle -> TargetHover -> AxisCreate/Edit -> StopSelect/Drag -> Commit/Cancel.

# Interaction

Uses the same spatial grammar as Gradient Tool while remaining visibly distinct. Stop edits affect opacity only.

# Context

Type, spread, selected stop opacity, invert and target appearance entry.

# Commands

AddTransparency, SetTransparencyGeometry, AddOpacityStop, SetOpacityStop, RemoveOpacityStop, InvertTransparency.

# Composition

Object opacity and transparency multiply/combine exactly as compositor contract specifies. UI never merges the concepts into one ambiguous value.

# Export

Adapter may preserve, convert to mask, flatten/expand or rasterize through explicit degradation plan.

# Tests

Nested groups, object opacity interaction, masks, alpha extremes, undo and export.