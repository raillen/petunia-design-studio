# 08.14 — Accessibility, Keyboard, Pointer, Pen, IME, Localization, HiDPI & Adaptive Density

# Amendment 2026-09-21 — Slint Accessibility

Replace GPUI-specific accessibility assumptions with Slint/platform accessibility capability. Petunia semantic metadata remains authoritative. Missing Slint/platform support must be tracked as a release gap or explicit platform limitation; it cannot silently delete keyboard, focus or accessible-name requirements.

# Accessibility is architectural

Every interactive component exposes semantic role, label, value/state, enabled/disabled, focusability, actions and bounds. GPUI/GPUI Kit accessibility support is used where available; Aubrieta semantic metadata remains explicit.

# Keyboard navigation

Required:

- logical Tab/Shift+Tab order inside dialogs/panels;
- arrow navigation in menus, trees, lists, grids and segmented controls;
- Enter/Space activation according to control type;
- Esc cancellation/dismissal hierarchy;
- visible focus ring independent of hover;
- command shortcuts not dependent on pointer.

Canvas itself has focused keyboard command model and can expose selection summary to accessibility layer.

# Focus management

Opening popover/dialog moves focus predictably. Closing restores focus to invoker when valid. Docking/reparenting panels must not unexpectedly drop keyboard focus.

# Screen readers

High-value labels include tool name, layer/object type, lock/visibility state, selected count, field units and error descriptions. Icon filename is never accessibility label.

Layers semantic example: `Flower Illustration, vector group, selected, visible, unlocked`.

# Color/contrast

Selection, error and active states use shape/icon/text changes in addition to color. Provide sufficient contrast in all themes. Canvas overlays can have alternate high-contrast palette.

# Reduced motion

System setting respected by default. Aubrieta preference can reduce additional decorative motion. Essential spatial motion becomes instant or fade-based.

# Pointer

Hover is enhancement, not requirement. Secondary click/context menu consistent. Drag thresholds prevent accidental reorder. Cursor updates semantically.

# Trackpad

Support smooth scrolling, pinch zoom, pan gestures and platform-appropriate inertial behavior without stealing gestures from controls under pointer.

# Pen/tablet

Photo painting needs pressure/tilt data where platform stack exposes it. Pressure/tilt support is **V1 Required where the platform exposes it**. Custom pen barrel/eraser remapping is a **Post-V1 Candidate**; V1 must still respect platform-standard pen/eraser identity without breaking pointer semantics. Palm rejection primarily platform responsibility; Aubrieta must not interpret touch/pen conflict unpredictably.

# IME

Text fields and creative text editing support composition correctly:

- preedit string;
- candidate positioning;
- commit/cancel;
- CJK/complex input;
- no global shortcuts firing during composition.

This is a release-blocking criterion for international text workflows.

# Localization

All UI strings externalized. Design for expansion:

- labels may grow 30–50%;
- no fixed pixel width based solely on English text;
- menus/panels can reflow;
- abbreviations used only when domain-standard;
- tooltip/help text localizable.

Numbers, decimal separators, dates and units respect locale where appropriate, while file formats/technical syntax remain invariant when specification requires.

# RTL

Shell should not assume leading=left in semantic layout where feasible. Canvas coordinates and creative conventions remain document-semantic. Text editing must honor BiDi through typography stack.

# High DPI

All icons vector or high-resolution assets. 1 px separators resolve to physical pixel where possible. Canvas handles, cursors and hit areas scale independently from document zoom.

Window movement between monitors with different scale factors updates UI, cursor, canvas rendering and floating panel geometry seamlessly.

# Adaptive window width

Desktop interface is not mobile-responsive, but must degrade gracefully:

- toolbar items move to overflow;
- panel minimums respected;
- tabs overflow/scroll;
- right dock may collapse;
- modal dialogs reflow vertically;
- no control overlaps.

The main-window minimum is a Design System/shell conformance value and must be defined before UI freeze, tested at 100/150/200% scaling and localized labels. Below the supported minimum, controlled overflow/collapse is preferred to overlapping controls; no feature panel may invent its own incompatible global minimum.

# Density and accessibility

Density mode changes visual metrics but never makes text illegible or keyboard focus unreachable. OS font/accessibility scaling overrides aesthetic preference when conflict arises.

# Localization QA matrix

At minimum test English, Brazilian Portuguese, German-length expansion, Arabic/RTL sample, Japanese/CJK IME. This is a UI layout gauntlet even before complete product localization ships.