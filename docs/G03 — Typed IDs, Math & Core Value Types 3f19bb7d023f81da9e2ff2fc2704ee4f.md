# G03 — Typed IDs, Math & Core Value Types

# Goal

Implement strongly typed IDs and foundational math/value types used across core.

# Depends

G01.

# Authority

09.2, 09.6.1, 09.6.2.

# Owner

systems-architect + implementer. Skills lang-cpp, clean-code, testing-quality.

# Deliverables

DocumentId/ObjectId/SurfaceId/ResourceId/etc wrappers; Vec2d/Rectd/Affine2D; finite-number validation; serialization helpers; equality/hash; UUID generation policy.

# Acceptance

Compile-time prevents mixing ID categories where practical; transformations/inverse/property tests pass; NaN/Inf rejected; deterministic string roundtrip.

# Performance

Value types trivially movable/copyable where appropriate; benchmark hashing/ID maps.

# Evidence

Unit/property tests, sanitizer clean.