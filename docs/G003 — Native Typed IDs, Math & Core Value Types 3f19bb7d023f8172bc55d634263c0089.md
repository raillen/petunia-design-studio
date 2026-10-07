# G003 — Native Typed IDs, Math & Core Value Types

# Goal

Implement foundational value types used by all C++ modules.

# Depends

G001.

# Primary

systems-architect + implementer. Review: quality-reviewer.

# Skills

lang-cpp, clean-code, memory-management, testing-quality.

# Deliverables

Strong types DocumentId/ObjectId/SurfaceId/ResourceId/StyleId/SymbolId/StoryId/NodeId/TransactionId/JobId; UUID generation/parsing/serialization; Vec2d/Rectd/Size/Insets; Affine2D; angle/unit helpers; finite-number validation; stable hashing where required.

# Invariants

Different ID kinds do not implicitly convert. Identity is not pointer/index. Math types contain no Qt classes. NaN/Inf rejected at canonical boundaries.

# Acceptance

Core headers compile standalone; equality/hash/string roundtrip stable; affine compose/invert and bounds helpers pass property tests.

# Tests

UUID roundtrip/invalid input; type non-convertibility compile checks; transform inverse/compose; extreme finite coordinates.

# Non-goals

Path geometry, document hierarchy, color engine.