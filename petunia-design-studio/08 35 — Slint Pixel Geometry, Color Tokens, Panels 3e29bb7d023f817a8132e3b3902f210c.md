# 08.35 — Slint Pixel Geometry, Color Tokens, Panels, Icons & Microinteraction Contract

# Coordinate model

All metrics are logical pixels. Slint scaling must preserve geometry at 100/125/150/175/200%. 1 px separators should map to crisp physical pixels where feasible without breaking layout.

# Global grid

Base 4 px. Allowed common spacing: 0, 4, 8, 12, 16, 20, 24, 32, 40. Half-step optical exceptions 2 px and 6 px are allowed only for icons/hit areas/separators.

# Dark theme reference palette

Petunia-owned target tokens:

- surface.workspace: #202124;
- [surface.chrome](http://surface.chrome): #27282B;
- surface.chromeStrong: #222326;
- surface.panel: #303236;
- surface.panelRaised: #373A3F;
- surface.control: #3A3D42;
- surface.controlHover: #464A50;
- surface.controlPressed: #50545B;
- border.subtle: #41444A;
- border.strong: #555A63;
- text.primary: #F2F3F5;
- text.secondary: #C2C6CC;
- text.tertiary: #8E949D;
- text.disabled: #666C75;
- accent.bloom: #B77AFF;
- accent.bloomHover: #C48FFF;
- accent.bloomPressed: #9F5FEA;
- accent.selectionSoft: rgba(183,122,255,0.20);
- [studio.design](http://studio.design): #35C7D4;
- [studio.photo](http://studio.photo): #F06C8D;
- status.success: #55C58A;
- status.warning: #E3AE52;
- status.error: #E76573.

Exact values may be tuned through visual QA, but feature code cannot hard-code alternatives.

# Typography

Caption 11/14; Small 12/16; Body 13/18; Body Medium 13/18 weight 500; Panel Title 13/18 weight 600; Dialog Section 15/20 weight 600; Dialog Title 18/24 weight 600. Use tabular figures for transform, dimensions, opacity, percentages and color channels.

# Control radii

3 px micro tags; 5 px fields/buttons; 7 px popovers/small floating surfaces; 10 px dialogs/floating palettes. Persistent dock boundaries remain square.

# Top chrome

Menu row 28 px; Persona row 40 px; context toolbar 34 px; tab strip 30 px. Borders are 1 px. Toolbar internal padding 4 px vertical and 8 px horizontal.

# Tool rail

Width 44 px; inner padding 5 px; 34×34 buttons; 18–20 px glyph; 2 px inter-button gap; semantic separators with 6 px margin. Hit area may extend beyond glyph, never below 32 px.

# Right dock

Default 304 px. Panel tab row 32 px. Panel inner padding 8 px. Section header 28 px. Property row 30–32 px. Search/filter row 30 px. Footer 32 px where used.

# Layer tree

Row 28 px; disclosure 16; type/thumbnail 20; indent 16/level; eye and lock hit targets 28; 4 px gaps. Rename field fills label region. Drag insertion line 2 px accent. Auto-scroll threshold 24 px from edge.

# Numeric field

Height 28 px Compact / 32 px Comfortable. Label scrubbing supported where meaningful. Click edits; double-click selects value; Enter commits; Esc reverts current edit; Up/Down step; Shift fine; Alt/Option or platform mapping only if contract defines it. Invalid values show border + message; no silent NaN.

# Tooltip

Initial delay 500 ms; subsequent nearby tooltip 100–150 ms. Padding 8 px; radius 7; max width 320. First line: title + shortcut; second: concise behavior. Disabled control tooltip may explain precondition.

# Menus

Row 28 px; icon column 20; label flexible; shortcut trailing; submenu chevron 12. Horizontal padding 8. Separator 1 px with 4 px vertical margin. Keyboard Up/Down/Left/Right/Enter/Esc. Submenu hover delay ~180 ms.

# Popovers

Border 1; radius 7; padding 8; modest shadow; close on outside click/Esc unless pinned; clamp to screen/work area. Popover focus returns to invoker on close.

# Modal

Backdrop dim; modal minimum 360 px. Common 480–720 px depending task. 20 px outer padding; 16–24 px section gap; actions 32 px high. Enter only triggers safe primary action; Esc cancels when allowed.

# Splitter

1 px visible, 6 px interaction. Hover uses accent soft. Drag live. Double-click reset preferred ratio. Keyboard resizing through accessibility/action command where feasible.

# Scrollbars

8–10 px visual track/thumb depending platform/density. Minimum thumb length 24 px. Must remain discoverable while actively scrolling. Nested scroll regions route wheel to the closest scrollable under pointer/focus.

# Icons

Semantic IconId only. General UI source: Lucide/Tabler normalized to Petunia stroke weight. Custom SVG for Pen/Node/Contour/Corner/Boolean/Stroke Width/Artboard/Gradient/Transparency and other domain tools where general icon libraries are ambiguous. Standard inline 16 px; toolbar 18; tool rail 20; dialog hero max 32.

# Cursor semantics

Arrow default; Hand/pan; Zoom; crosshair for precision draw/cut; text caret; node-edit; eyedropper; resize splitters; forbidden drop; rotate/scale/transform semantic cursors. Cursor asset IDs are tokenized and DPI-aware.

# Persona control

Design/Photo segmented control sits near brand mark. Segment height 30 px within 40 px row. Active Design can use [studio.design](http://studio.design) cyan to preserve immediate mode recognition while global selection/focus uses Bloom purple. Photo uses [studio.photo](http://studio.photo) only for Persona identification, not arbitrary photo-panel decoration.

# Microinteraction timing

Hover 80–120 ms; press immediate; menu/popover 120–160; panel collapse 140–180; dialog 160–220. Reduced Motion removes translation/scale. No spring/bounce chrome.

# Focus

2 px focus ring with 1 px offset when geometry allows. Focus state must be visible in dark/light themes and cannot reuse hover as the only indication.

# Dirty/working feedback

Document tab dirty dot is persistent. Long jobs appear in status/background tasks. Busy action never accepts duplicate activation if non-idempotent. Save failure remains visible until resolved; it is never replaced by a transient success-looking state.