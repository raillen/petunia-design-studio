# G028 — First PTND Vector Golden Workflow

# Goal

Prove end-to-end architecture using Demo A subset before expanding engine breadth.

# Depends

G001–G027 applicable.

# Primary

tester + editor-engineer; release-style verifier independent.

# Workflow

Launch -> New -> Surface -> Rectangle/Ellipse -> Fill/Gradient/Stroke -> Move/resize/rotate -> Layers rename/group baseline if ready -> Undo/Redo -> Save PTND -> close -> reopen -> Export PNG/SVG.

# Deliverables

Golden .PTND fixture; scripted semantic UI interaction; normalized canonical snapshots; PNG visual golden; SVG structural golden; performance trace; bug list.

# Acceptance

No semantic drift after reopen; no mutation outside Commands; UI/shortcut Action parity; output validated; accessibility critical path keyboard-operable; budgets measured and major blockers fixed/recorded.

# Evidence

Single reproducible bundle with BuildId/commands/screenshots/snapshots/timings.

# Non-goals

Claim commercial V1; Pen/Node paths; raster Photo.