# 18.18 — Frame Text Tool Specification

# Creation

Drag rectangle creates TextFrame + Story when standalone. Clicking existing frame enters caret. Optional click place/auto-size behavior only if explicit.

# Geometry

Frame bounds, insets, columns, vertical alignment, auto-size modes and text flow ports.

# Resize

Move/frame handles resize container without changing font size. Overset indicator updates derived.

# Linking

Click output port enters LinkTextFrame state; click target compatible frame links; drag empty creates new linked frame if workflow enabled. Esc cancels.

# Context

font basics plus frame columns/insets/vertical align/auto-size and flow state.

# Text wrap

Frame layout consumes external wrap objects; tool may expose wrap visualization but wrap settings live panel/Properties.

# Commands

CreateTextFrame, SetFrameGeometry, LinkTextFrames, UnlinkTextFrames, EditTextRange.

# Tests

Columns, overset, linked flow, resize, IME, bidi, text wrap and save/reopen.