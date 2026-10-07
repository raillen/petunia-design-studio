# 18.14 — Artistic Text Tool

# Identity

ToolId ptnd.tool.artistic_text. Text family shortcut T.

# Creation

Click canvas creates point text at insertion origin and enters text edit. Optional drag may establish initial scale/size behavior if product enables; semantics must not confuse with Frame Text.

# Editing

Click existing ArtisticText enters caret. Dragging with Move Tool transforms object; font size remains semantic unless user chooses Scale Text behavior.

# Context

font family/style, variable axes quick access, size, color, basic OpenType, alignment where meaningful.

# Commit/history

Creating object + first typing can be grouped by creation session or separated according history UX policy; text typing coalesces independently.

# Empty object

If user creates then exits without text, delete empty object automatically only under documented predictable rule.

# Conversion

Convert to Curves explicit and warned as loss of editability/search/accessibility.

# Accessibility

Native text caret/selection semantics exposed; full keyboard editing.

# Tests

IME, bidi, variable fonts, transform, empty creation, undo creation+typing and save.