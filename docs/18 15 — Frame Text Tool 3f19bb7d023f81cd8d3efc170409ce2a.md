# 18.15 — Frame Text Tool

# Identity

ToolId ptnd.tool.frame_text.

# Creation

Drag rectangle creates TextFrame bound to new/existing TextStory according flow operation. Enters caret at start.

# Frame handles

Resize frame changes container geometry, not font size. Insets/columns and auto-size settings in Properties.

# Overflow

Overset indicator shown when story content remains after frame capacity. Clicking overflow port begins link/create-next-frame state.

# Linking

Output port -> existing eligible frame or drag new frame. Flow graph validates no cycle. Input port allows navigating previous flow.

# Context

font basics, columns quick field, vertical alignment, auto-size, frame fill/stroke shortcuts if supported.

# Commands

CreateTextFrame, SetFrameGeometry, LinkTextFrames, UnlinkTextFrames, SetFrameProperties, EditTextRange.

# Accessibility

Frame relation/order announced; overflow status exposed.

# Tests

Columns, linked frames, resize, overset, cycle rejection, deletion of middle frame, IME, save/export.