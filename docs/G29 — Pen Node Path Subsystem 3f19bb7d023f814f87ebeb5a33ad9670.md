# G29 — Pen/Node Path Subsystem

# Goal

Implement professional editable cubic path creation/editing on geometry core.

# Depends

G28, geometry 09.6.*.

# Authority

18.02–18.03, 10.2.

# Owner

editor-engineer + systems-architect.

# Deliverables

VectorPath/Contour/Node schema, curve math, Pen state machine, Node tool, insert/delete/join/break/reverse, renderer path primitive, PTND serialization and SVG export.

# Acceptance

One path creation undo step; stable NodeIds; exact insertion; complex path save/reopen; snap endpoints/tangents; fuzz and visual tests.

# Evidence

curve corpus, sanitizer/fuzz, tool interaction manifest.