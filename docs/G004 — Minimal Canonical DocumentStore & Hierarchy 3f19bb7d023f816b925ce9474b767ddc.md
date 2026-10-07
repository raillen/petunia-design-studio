# G004 — Minimal Canonical DocumentStore & Hierarchy

# Goal

Implement canonical document ownership, Surface/object hierarchy and typed lookup without UI/render concerns.

# Depends

G003.

# Primary

architect + editor-engineer.

# Skills

lang-cpp, architecture-quality, editor-tooling, clean-code, testing-quality.

# Deliverables

DocumentStore; Document metadata; Surface baseline; common ObjectRecord variants Group and ParametricRectangle placeholder; resource/style registries skeleton; stable typed lookup; parent/child ownership; insertion/removal/reparent primitives internal to mutator; immutable snapshot/query API.

# Invariants

No cycles; every owned ObjectId unique; parent/Surface relation valid; no Qt/Python/backend types; external callers cannot mutate containers directly.

# Acceptance

Create document/Surface/group/rectangle; query by ID; snapshot; validate; remove/reparent safely.

# Tests

Cycle rejection, duplicate ID, invalid parent, deletion subtree semantics, snapshot immutability and 100k-object lookup baseline.

# Non-goals

Public Command mutation path (G005), rendering, serialization full schema.