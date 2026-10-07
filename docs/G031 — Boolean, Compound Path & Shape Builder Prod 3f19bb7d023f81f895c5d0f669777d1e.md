# G031 — Boolean, Compound Path & Shape Builder Production Geometry

# Goal

Deliver robust Boolean/compound/Shape Builder on top of path engine.

# Depends

G029, geometry 09.6.2–09.6.5.

# Primary

systems-architect + editor-engineer.

# Deliverables

IBooleanEngine production path; Union/Subtract/Intersect/XOR/Divide; LiveBoolean node; Compound Path; Shape Builder region engine/tool; provenance/ID remap; style policy; fuzz corpus.

# Acceptance

Complex holes/self-intersections/coincident-edge fixtures behave deterministically, undo/save/export correctly and never partially mutate on failure.

# Tests/Evidence

Differential corpus, fuzz, renderer goldens, 10k-segment benchmark, UI/MCP action parity.

# Non-goals

Mesh gradients or generic CAD topology.