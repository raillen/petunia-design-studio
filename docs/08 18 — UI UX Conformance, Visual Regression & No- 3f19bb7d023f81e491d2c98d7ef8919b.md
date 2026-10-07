# 08.18 — UI/UX Conformance, Visual Regression & No-Gap Coverage Ledger

# Evidence

Feature UI requer semantic interaction test, keyboard path, visual golden quando aplicável, alternate theme/density, DPI profile, locale expansion e accessibility review para component types novos.

# Golden profiles

Windows 100/150%, Linux Wayland 100/200%, macOS Retina; Dark/Light; Comfortable/Compact.

# Semantic golden

IDs, roles, labels, enabled/visible, action IDs, focus order e critical geometry.

# Interaction manifest

Tool activation, shortcuts, context controls, modifiers, gestures, commit/cancel, panel dependencies e expected ActionIds.

# Coverage ledger

Compare current Affinity capabilities/workflows com Petunia status, sem pixel cloning.

# Release blockers

- unlabeled control;
- mouse-only essential action;
- stale context/tool state;
- missing cancel on long job;
- hidden export degradation;
- panel direct mutation outside Command;
- hard-coded colors/metrics outside tokens;
- broken mixed-DPI restore.