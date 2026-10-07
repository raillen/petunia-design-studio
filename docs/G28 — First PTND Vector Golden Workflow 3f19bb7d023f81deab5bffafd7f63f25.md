# G28 — First PTND Vector Golden Workflow

# Goal

Prove the architecture works end-to-end before broadening scope.

# Depends

G09–G27.

# Authority

17.5 Demo A.

# Owner

tester + editor-engineer; independent quality/accessibility review.

# Workflow

New document → Surface → rectangle/ellipse → transform → fill/gradient/stroke → Layers rename/group → undo/redo → save → close/reopen → export PNG/SVG.

# Acceptance

Semantic snapshot before save equals reopen; UI/keyboard core flow passes; no critical sanitizer errors; provisional performance budgets recorded.

# Evidence

Golden PTND, screenshots/semantic tree, exported files, test commands.