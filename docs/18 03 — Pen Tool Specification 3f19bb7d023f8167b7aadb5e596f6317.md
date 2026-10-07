# 18.03 — Pen Tool Specification

# Identity

ToolId ptnd.tool.pen; default shortcut P. Modes: Pen, Smart candidate, Polygon.

# State machine

Idle → FirstAnchorPlaced → BuildingContour → DraggingNewHandle | AdjustingPreviousHandle | ClosingHover → Finished/Cancelled.

# Input

Click creates corner; drag creates smooth handles. Shift constrains angle; Alt/Option breaks handle relation. Preview segment follows pointer.

# Closing and finishing

Hover start node shows close affordance; click closes. Enter finishes open path. Esc cancels current uncommitted stage first; subsequent Esc follows configured finish/cancel policy.

# Continue

Click highlighted endpoint of compatible open path to continue. Joining two existing paths follows Join semantics.

# Context

mode, node behavior, fill/stroke, snapping and preview options.

# History

Path creation appears as one logical Create Path transaction even if internal recovery checkpoints exist.

# Core boundary

Python handles interaction state; C++ handles geometry, snapping and path mutation.

# Accessibility and tests

Finish/cancel/modes keyboard-accessible; exact editing via Node/Properties. Test click/drag/cusp/close/continue/lost capture/tool switch and undo.