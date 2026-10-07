# 18.35 — Blemish Removal Tool Specification

# Purpose

Fast spot/short-stroke healing for small defects.

# Interaction

Click creates circular repair region sized by brush; short drag creates elongated region. Automatic source suggestion visualized and optionally repositionable before commit if latency allows.

# Algorithm

Can reuse HealingEngine with auto-source selector. Source selection must avoid target overlap and respect image boundaries/masks.

# Context

size, hardness/feather, auto/manual source, sample scope.

# Commit

One repair operation transaction per click/stroke with tile delta.

# Tests

spots near edges, repeated clicks, textured background, source fallback and undo.