# G033 — Text Canonical Model & Editing Command Kernel

# Goal

Introduce TextStory, runs, paragraph spans, ranges and text-edit Commands independent from renderer/Qt.

# Depends

G003–G007, 09.9.1/09.9.4.

# Primary

editor-engineer + systems-architect.

# Deliverables

Story storage candidate benchmark/selection; Unicode index abstraction; run/style inheritance; splice edit; selection/caret mapping primitives; history deltas; PTND schema.

# Acceptance

Unicode text edits/style ranges roundtrip and undo exactly with no byte-index assumptions.

# Tests

Graphemes, ZWJ, combining marks, huge story edits, style split/merge, property fuzz.