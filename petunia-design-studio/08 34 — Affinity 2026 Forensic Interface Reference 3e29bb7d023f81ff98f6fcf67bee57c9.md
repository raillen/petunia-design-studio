# 08.34 — Affinity 2026 Forensic Interface Reference → Petunia Slint Translation

<aside>
🔎

**Reference status:** the 2026 Affinity Vector Studio lesson supplied by the project owner is prior art for composition, hierarchy and interaction density. Petunia Design Studio must not copy Affinity/Canva branding, proprietary icons or trade dress. We translate the observed ergonomic structure into Petunia-owned tokens/components and the existing Design/Photo Persona model.

</aside>

# Reference observations

The current Affinity desktop UI uses a compact dark professional shell with multiple horizontal command layers: a classic application menu row, a high-salience Studio selector row, a context/tool-options row, document tabs, a central canvas, a narrow left tool rail, dense right-side Studio panels and a low-height status strip. The active Vector Studio is represented by a bright cyan rounded segment against charcoal chrome. Persistent panels are mostly flat/docked; temporary surfaces are more elevated.

# Petunia translation

Petunia keeps the same high-level information hierarchy but uses **Design / Photo** as the top Persona switch. Layout is a Design workflow/profile, not a third canonical Persona. A Workspace Profile dropdown may expose Illustration, Layout, Print, Photo Retouch and Custom without changing document semantics.

# Desired desktop anatomy at 100% logical scaling

1. Native/window frame: platform-owned where applicable.
2. Menu bar: 28 px target.
3. Brand + Persona/Workspace bar: 40 px target.
4. Context toolbar: 34 px target.
5. Document tab strip: 30 px target.
6. Main editor region: remaining height.
7. Status bar: 26 px target.

These values are Petunia implementation targets derived from the visual density of the reference; they are not claims about Affinity source metrics.

# Horizontal editor anatomy

- Left tool rail: 44 px default; 40 px Compact; 48 px Spacious.
- Optional left dock: 240–340 px preferred depending panel.
- Central canvas: flexible and always the dominant region.
- Right dock: 304 px default; preferred 280–360; resizable 240–520.
- Bottom dock: hidden by default in Design; 180–280 px when History/Assets/Jobs are docked.
- Splitter visible stroke: 1 px; hit target 6 px.

# Menu bar

Top-level: File, Edit, Object, Layer, Select, Text, View, Window, Help. Photo may contextually contribute Image and Filter while preserving ordering rules. Each row is ActionRegistry-driven. No menu callback contains feature logic.

# Brand/Persona row

Leading cluster: 24 px Petunia flower mark inside a 32 px hit area, then optional application title only on home/no-document state. Persona selector is a compact segmented capsule: Design and Photo. Active segment receives strong filled accent plus icon/text state; inactive segments remain quiet. Immediately after it: Workspace Profile dropdown and optional capability chips. Trailing area: Undo, Redo, snapping/display quick controls, Export, command palette/search and overflow depending width.

# Context toolbar

Exactly one row. It reflects active tool + selection. Tool families keep stable control positions. Numeric controls use consistent widths and units. Advanced controls overflow into a popover instead of growing a second toolbar row. Empty tool context must not create a blank-looking bar: it may show selection summary or be visually quieter while retaining height.

# Document tabs

Tabs begin at central editor x-origin rather than under the left tool rail. Each tab shows document name, dirty indicator and close affordance. Active tab has a subtle lighter graphite fill and stronger text. Modified state remains visible even without hover. Overflow scroll/menu appears before labels become unreadable.

# Left tool rail

Dark flat rail, icon-only, compact grouping, strong active-tool state. Standard glyph 18–20 px; button 34×34 Comfortable; group marker 6 px max in lower-right corner; separators between semantic groups. Tool group opens on right-click or long press; click on last-used tool activates directly. Tooltip: title, shortcut, one-sentence purpose.

# Canvas

Pasteboard is darker than paper/surface and slightly darker/lighter than panel chrome enough to establish depth. White/colored Surface remains content focus. Selection overlays use Petunia semantic overlay tokens, not UI accent blindly. Rulers are optional by View state; guides/snaps remain visible over varied artwork through contrast-aware rendering.

# Right Studio panels

The Affinity reference heavily relies on tabbed panel stacks such as Colour/Swatches/Stroke/Appearance and Layers/Path/Brush/Quick FX/Styles, with Transform/Navigator/History beneath. Petunia adopts the **stacked dock mental model**, not those exact proprietary groupings. Design Default right dock:

- Stack A: Color | Swatches | Stroke | Appearance;
- Stack B: Layers | Properties;
- Stack C: Transform | Align | Navigator | History;

Collapsed/hidden panels remain available through Window → Panels and command palette.

# Color panel reference translation

Panel header tabs around 30–32 px. Body can show dual Fill/Stroke swatches, swap/default/none, model selector, wheel/triangle or sliders, channel values, hex when meaningful and opacity. Wheel must never become the only precise entry path. CMYK/Lab/Gray use appropriate controls rather than RGB cosplay.

# Layers panel reference translation

Top compact row: Opacity + Blend Mode + contextual panel actions. Tree rows 28 px Comfortable. Anatomy: disclosure 16, thumbnail/type 20, name flexible, badges, lock and eye. Selected row uses tinted background plus stronger label/icon state. Hover reveals secondary actions without shifting label geometry. Nested indentation 16 px per level.

# Bottom/right precision panels

Transform uses tabular numeric figures and compact two-column/property-grid layout. Navigator shows document thumbnail with viewport rectangle and zoom. History uses transaction rows and current marker; no time-travel semantics beyond the actual History contract.

# Visual direction

Reference qualities to preserve: restrained graphite chrome, low visual noise, compact professional density, one bright active-workspace accent, flat docked surfaces, subtle 1 px separators, small-radius controls, semibold only where hierarchy needs it, no card-heavy consumer-dashboard look.

# Petunia identity divergence

Petunia adds its own botanical identity through the logo, Bloom accent family, icon grammar and micro-brand moments in onboarding/About only. Editing chrome stays neutral. The product must feel like a serious design tool, not a themed app.

# What we explicitly do not copy

Affinity logo, Canva AI pill, Affinity wordmarks, exact proprietary icons, exact branded gradients, marketing names, screenshots as shipping assets, or unmodified panel arrangements presented as Petunia originals.

# Reference cross-check — 2026-09-21

The supplied direct MP4 host did not expose a fetchable media stream to this environment, so pixel observations must be treated as **implementation targets derived from the supplied reference context plus current official/public Affinity material**, not as source-code measurements of the video.

Current Canva Design School material updated 17 Sep 2026 explicitly teaches the Affinity UI, Studios, Layers, panels/toolbars and customizable Studio layouts:

- [https://www.canva.com/design-school/lessons/welcome-to-affinity/](https://www.canva.com/design-school/lessons/welcome-to-affinity/)
- [https://www.canva.com/design-school/resources/customize-your-affinity-studios/](https://www.canva.com/design-school/resources/customize-your-affinity-studios/)

Affinity workspace documentation also establishes the enduring hierarchy Menu bar → Persona/Studio switch → Toolbar → Context Toolbar → Tools → Studio panels → Status bar. Petunia deliberately translates this hierarchy into **Design/Photo Persona + Workspace Profile + context toolbar + tool rail + dock panels + status bar**, rather than copying Affinity branding.

The exact Petunia dimensions/tokens in 08.34/08.35 are therefore **Petunia-owned implementation specifications** selected to reproduce the observed professional density and ergonomics; they are not claims about proprietary Affinity internal pixel constants.