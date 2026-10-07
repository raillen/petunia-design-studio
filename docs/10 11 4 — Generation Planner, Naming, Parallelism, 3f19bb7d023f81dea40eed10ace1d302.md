# 10.11.4 — Generation Planner, Naming, Parallelism, Cancellation & Deterministic Outputs

# Plan

Snapshot template revision + source fingerprint + BindingSet + records + output mode + export presets + naming template.

# Preflight

Validate schema/types, grants, filename collisions, resources/fonts, overflow and export fidelity.

# Naming

Safe expression subset; sanitize reserved/path characters, enforce length, prevent traversal and pre-detect collisions.

# Parallelism

Bounded worker batches; deterministic output order; parallel render/export limited by memory/renderer budget.

# Cancellation

Stop scheduling new records; in-progress outputs commit atomically or discard temp; completed outputs remain valid.

# Report

Per-record success/skipped/failed, outputs, warnings, degradations and duration.

# Tests

100k dry-run, duplicate names, cancel at N, deterministic ordering, stale source and partial cleanup.