# G06 — Native Error Model & nanobind Boundary

# Goal

Expose a safe coarse-grained native facade to Python.

# Depends

G03–G05.

# Authority

09.4, 04.3, 12.1–12.2.

# Owner

systems-architect + implementer.

# Deliverables

petunia_native module; ApplicationCore/DocumentSession minimal facade; typed exceptions/result codes; ID/value bindings; GIL release example; event/completion bridge skeleton.

# Acceptance

Python cannot receive dangling child pointer; deleted ID returns NotFound; native error maps to typed Python exception; long native test releases GIL; interpreter shutdown sanitizer test passes.

# Non-goals

No Qt widgets or fine-grained geometry bindings.