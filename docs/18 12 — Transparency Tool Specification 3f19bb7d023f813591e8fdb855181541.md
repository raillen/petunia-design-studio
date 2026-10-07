# 18.12 — Transparency Tool Specification

# Purpose

Edit object/appearance opacity gradients without conflating transparency with fill color.

# Model

OpacityGradientDefinition shares geometry with gradient but stop value is scalar opacity.

# Interaction

Same on-canvas grammar as Fill/Gradient. Selected stop shows opacity. Invert/clear explicit.

# Composition

Mask multiplies alpha at compositor-defined stage. Interaction with object opacity/group masks follows 09.8.2.

# Commands

AddTransparencyMask, SetTransparencyGeometry, Add/Move/DeleteOpacityStop, SetOpacityStop, RemoveTransparencyMask.

# Accessibility/tests

Numeric stop editor and command actions. Test nested opacity, group/mask interaction and save/export.