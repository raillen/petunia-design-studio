# 18.34 — Inpainting Tool Specification

# Purpose

Remove marked region and synthesize replacement using built-in deterministic/local algorithm or explicit provider.

# Marking

Brush defines inpaint mask; target/source context captured at request revision.

# Job

Potentially expensive: snapshot → compute cancellable native/provider job → staged patch → revision validation → atomic commit.

# Provider

Network/AI provider, if ever supported, requires separate permission/privacy UI and cannot be silently used. Offline built-in remains distinct capability.

# Parameters

search radius/quality/structure preservation only if algorithm makes them meaningful. Avoid fake sliders.

# Progress

Background progress and Cancel. Canvas may preview mask while computation runs.

# Failure

No partial tile mutation. Error preserves selection/mask for retry.

# Tests

simple removal fixtures, large region cancellation, stale revision, offline guarantee, alpha and undo.