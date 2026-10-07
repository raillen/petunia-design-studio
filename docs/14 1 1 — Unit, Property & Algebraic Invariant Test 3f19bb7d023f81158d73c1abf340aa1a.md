# 14.1.1 — Unit, Property & Algebraic Invariant Test Matrix

# Unit

Small deterministic tests for value types, parsing, commands, schemas and isolated algorithms.

# Property tests

Geometry: transform inverse, reverse twice identity, insert cubic preserves curve, boolean consistency properties where valid.

Raster: COW isolation, tile coordinate roundtrip, selection combine algebra.

Text: splice/run coverage, index mapping, style range invariants.

Color: identity transforms, clamp/domain invariants.

History: command + undo returns semantic snapshot.

# Generators

Bounded random paths, transforms, Unicode strings/runs, pixel tiles, nested hierarchies and command sequences. Generator constraints recorded so failures reproduce by seed.

# Shrinking

Property test framework must shrink to minimal failing example and persist regression fixture for serious defects.

# Floating point

Assertions use domain-specific tolerances; no global approximate equality helper for all subsystems.

# CI

Fast property subset on each PR, deeper seeded/nightly corpus and saved failure seeds.