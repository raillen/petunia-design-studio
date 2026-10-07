# G038 — PDF Export Baseline & Multi-Page Professional Output

# Goal

Ship non-PDF-X professional PDF export baseline.

# Depends

G034–G037, color hooks, 09.12, 25.2.

# Primary

editor-engineer + security-architect for writer/parser boundary.

# Deliverables

PDF writer adapter; vector/text/image/clips/transparency subset; page boxes; font embedding/subsetting; ICC hooks; degradation analyzer; output validation.

# Acceptance

Brochure fixture exports multi-page PDF with correct page order, text/vector fidelity and explicit warnings.

# Tests

External viewers/inspectors, fonts, transparency, malformed options, atomic output.

# Non-goal

PDF/X certification (G058).