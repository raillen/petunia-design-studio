# 08.4.3 — Temporary Tool Overrides, Modifier Priority & Input Context Stack

# Input context stack

Priority: modal/dialog text input > active text editing > captured tool gesture > temporary tool override > global shortcut > tool activation.

# Space Hand

Press while compatible tool idle/gesture that permits pan pushes Hand override; release pops and restores exact prior tool state. If pointer captured for path drawing, policy defines whether pan can temporarily suspend without commit.

# Modifiers

Shift/Alt/Ctrl/Cmd are gesture modifiers when pointer operation active; they are shortcut chords otherwise. Status bar reflects interpretation.

# Stuck key

Window deactivation/focus loss clears temporary overrides/modifier state and emits CaptureLost where needed.

# Tablet buttons

Can map to temporary tool/action through same input binding system, not separate hard-coded code.

# Accessibility

Sticky-key OS behavior respected; do not require simultaneous modifier for essential command without alternative.

# Tests

Space during Pen/Brush/Move, app alt-tab mid-override, modifier precedence and remapped tablet button.