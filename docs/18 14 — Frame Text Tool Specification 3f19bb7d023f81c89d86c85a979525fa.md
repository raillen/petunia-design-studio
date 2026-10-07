# 18.14 — Frame Text Tool Specification

# Identity

ToolId ptnd.tool.text_frame. Persona Design.

# Creation

Drag creates frame geometry and associated TextObject/Story when no story is assigned. Click an existing frame enters text editing. A selected closed vector can be converted to/used as text container only through explicit action.

# Canonical

Frame stores geometry, insets, columns, vertical alignment, auto-size policy, text-wrap interaction and flow links. Story content is separate from frame placement.

# State machine

Idle -> DragFrame -> FrameCreated/TextEditing -> LinkPortArmed -> LinkPending -> Linked/Cancelled.

# Overlay

Frame boundary, inset/column guides, flow input/output ports, overset marker and text-wrap boundary where relevant.

# Link flow

Click output port then compatible frame to link stories. Clicking empty canvas can create next frame if enabled. Cycle prevention is mandatory.

# Context

Typography basics plus frame insets, columns/gutter, vertical alignment, auto-size, baseline-grid alignment and overflow options.

# Resize semantics

Move/resize handles change frame geometry only. Text size is never implicitly changed by frame resize.

# Commands

CreateTextFrame, SetFrameGeometry, EditTextRange, LinkTextFrames, UnlinkTextFrames, SetFrameLayoutProperties.

# Tests

Linked frames, page reorder, overset, columns, insets, bidi, IME, cycle rejection, undo and export.