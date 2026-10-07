# 07.3 — Determinism, Reproducibility & Golden Evidence Policy

# Deterministic domains

Geometry, serialization, command semantics, fixed-seed brush/effects, CPU reference rendering and schema validation should be deterministic for same inputs/version.

# Allowed variation

GPU raster/filter small numeric tolerance, platform font discovery/fallback, compression byte streams and OS window chrome. Semantic/normalized comparison handles these.

# Fixtures

Every critical capability has minimal and stress fixture with provenance/license. Randomized tests record seed on failure.

# Golden updates

Never auto-accept. Reviewer sees before/after diff, reason and linked behavior change.

# Reproducibility

Evidence includes BuildId, dependency locks, OS/GPU/driver, command, fixture and relevant settings.

# Artifact retention

Release evidence kept according project policy; failed CI visual/perf/fuzz artifacts retained sufficiently for debugging.