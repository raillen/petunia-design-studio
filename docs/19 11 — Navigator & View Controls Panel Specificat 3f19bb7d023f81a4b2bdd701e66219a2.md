# 19.11 — Navigator & View Controls Panel Specification

# Navigator

Derived thumbnail of current/all Surface, viewport rectangle, zoom field/slider and Surface selector if multiple.

# Interaction

Drag viewport rectangle pans; click thumbnail centers; wheel/keyboard zoom. Does not mutate document.

# Thumbnail

Async/rendered low-resolution; updates coalesced after document changes, not every brush dab individually beyond useful rate.

# Multiple Surfaces

Overview mode displays Surface bounds; user selects current Surface/view target.

# Proof/pixel preview

View toggles can live adjacent but remain view state.

# Accessibility/tests

Viewport actions available as commands/numeric zoom. Test huge canvas, rotated view if supported and independent multi-view state.