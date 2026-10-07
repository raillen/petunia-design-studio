# 20.5 — Preflight Rule Engine, Issue Codes, Fix Actions & Export Blocking Policy

# Rule

PreflightRule {

id/version

applies_to target preset/capabilities

severity default

query dependencies

evaluate(snapshot,target)

optional deterministic fix ActionId

}

# Core rules

Missing font/resource/profile, overset text, effective raster DPI low/high, RGB in CMYK-only target, unsupported spot/effect, transparency flattening, TAC, registration misuse, object outside/insufficient bleed, hidden overflow, export degradation and filename/output conflict.

# Issue

IssueCode, severity, ObjectId/SurfaceId, message TextId+params, technical details, navigation target, affected count and fix action if safe.

# Blocking

Preset maps issue/degradation categories to allow/warn/confirm/block. Strict PDF/X blocks standard violations.

# Incremental

Panel can preflight asynchronously and invalidate only relevant rules after ChangeSet.

# Fix

A fix is a normal Action/Command, previewable and undoable. Preflight never mutates document automatically.

# Tests

Rule determinism, issue navigation, stale result discard, fix action undo and strict-export blocking.