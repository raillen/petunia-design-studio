# G04 — Minimal Canonical DocumentStore

# Goal

Implement minimal canonical document hierarchy independent from UI/rendering.

# Depends

G03.

# Authority

01, 09.2, 09.27, PTND schema docs.

# Owner

architect + editor-engineer.

# Deliverables

Document, Surface, Group, RectShape minimal, object registry, parent/child ordering, visibility/lock/name/transform, resource/style placeholders and semantic validator.

# Invariants

Unique IDs, single ownership parent, acyclic hierarchy, stable paint order, no Qt/Python types.

# Acceptance

Create/remove/reparent objects headlessly; invalid cycles/dangling IDs rejected; snapshot semantic equality; ASan/UBSan clean.

# Non-goals

No commands/history yet; direct mutation only inside tests/internal builder until G05.