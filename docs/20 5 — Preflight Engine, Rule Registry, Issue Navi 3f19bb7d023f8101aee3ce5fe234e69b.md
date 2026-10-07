# 20.5 — Preflight Engine, Rule Registry, Issue Navigation & Fix Actions

# Architecture

PreflightEngine evaluates immutable document/export snapshot against RuleRegistry + target ExportProfile.

# Rule

RuleId, category, severity default, applicability predicate, parameters/thresholds, scanner, issue code, help topic and optional safe FixAction.

# Categories

Fonts, Resources, Color, Ink/Prepress, Geometry/Page, Text Overflow, Effects/Fidelity, Export Capability, Data Merge and File Output.

# Issue

IssueId, RuleId, severity, ObjectId/SurfaceId/resource refs, message parameters, evidence values and suggested actions.

# Navigation

Selecting issue focuses relevant Surface/object/panel without mutating document. Multiple objects can be highlighted.

# Fix

Only deterministic reversible fixes are offered. Fix invokes normal Action/Command and reruns affected rules. No hidden batch conversion.

# Incremental

Interactive panel may re-evaluate affected rules from ChangeSet; final export preflight runs complete snapshot.

# Headless

MCP/CLI receives same structured report and rule IDs.

# Tests

Rule determinism, incremental invalidation, fix/undo, multiple pages, suppression/profile settings and export parity.