# 18.33 — Healing Tool Specification

# Purpose

Source-based texture transfer with tone/color adaptation.

# Source

Manual Alt/Option source baseline; optional automatic source mode is separate algorithm.

# Pipeline

Sample source texture/patch → compute destination neighborhood characteristics → blend texture while matching low-frequency color/luminance. Algorithm versioned behind IHealingEngine.

# Brush/context

size, hardness, opacity/flow if meaningful, aligned, sample scope, diffusion/quality only if algorithm exposes stable semantics.

# Preview

Fast approximate during stroke if necessary; final result converges on pointer-up before commit or atomically replaces staged result.

# Color

Healing operates in defined working linear/perceptual domain and respects alpha/masks.

# Tests

skin patch fixtures, gradients, transparent edges, transformed source, 16-bit and CPU/GPU/algorithm-version stability.