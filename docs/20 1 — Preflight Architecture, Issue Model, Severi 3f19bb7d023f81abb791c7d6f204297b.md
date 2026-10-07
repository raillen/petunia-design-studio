# 20.1 — Preflight Architecture, Issue Model, Severity & Fix Actions

# PreflightService

Runs rule set against immutable DocumentSnapshot + export/output target. Returns PreflightReport revision-bound.

# Issue

IssueId/code; severity Error|Warning|Info; category; ObjectId/SurfaceId/resource refs; message TextId + parameters; evidence/detail; deterministic FixActionId optional; suppressibility policy.

# Categories

Document/setup, color/profile, fonts/text, raster resolution, transparency/effects, resources/links, bleed/marks, spots/overprint, PDF/export capabilities, Data Merge/output names.

# Severity

Error = output violates selected hard requirement or cannot export safely.

Warning = export possible but likely quality/fidelity issue.

Info = advisory optimization/awareness.

Rules can change severity by output preset (screen PNG vs PDF/X).

# Fix

Only expose automated fix if deterministic and reversible. Fix executes normal Action/Command and reruns affected rules. “Rasterize everything” is not generic auto-fix.

# Incremental

Interactive preflight can maintain derived issue index from ChangeSets. Final export preflight executes authoritative complete rules.

# Suppression

Per-code/document suppression only for suppressible warnings, stored with rationale if product supports; errors for conformance cannot be hidden into success.

# UI

Issue list grouped severity/category, search, navigate to object/page, details, Fix/Fix All safe set, rerun status.

# Tests

Issue stability, navigation after edits, stale report, preset severity changes, fix undo and final export blocking.