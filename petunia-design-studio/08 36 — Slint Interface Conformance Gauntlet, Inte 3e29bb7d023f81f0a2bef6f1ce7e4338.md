# 08.36 — Slint Interface Conformance Gauntlet, Interaction Manifest & Release Gates

# Zero fake UI

Every visible control is either functional, intentionally disabled with a real reason, or absent. Placeholder buttons are forbidden in release UI.

# Interaction Manifest fields

SurfaceId; component type; parent screen/panel; ActionId/ToolId; TextId; IconId; shortcut; enabled predicate; disabled reason; pointer behavior; keyboard behavior; focus; tooltip; state mutation; transient state; feedback; cancel; undo; persistence; accessibility; performance class; security class; automated test; manual proof; evidence status.

# Required component-state matrix

Default, hover, pressed, focused, disabled, selected/toggled, mixed/indeterminate, loading/busy, error, warning, long-localized label, Dark, Light, Compact, Comfortable, 100/150/200% scale.

# Required shell matrix

1366×768, 1600×900, 1920×1080, 2560×1440; continuous resize; maximized/restored; fullscreen; mixed-DPI monitor movement where test environment allows; missing-monitor workspace restore.

# Docking tests

Resize left/right/bottom; min/max; double-click reset; tab reorder; move stack; float/redock if enabled; Esc cancel drag; unavailable contributed panel; serialize/restart/restore; 20+ panel stress; keyboard panel command.

# Menu tests

Open every menu/submenu; keyboard navigation; pointer navigation; disabled predicates; shortcuts; no dead item; no command only reachable through context menu unless explicitly secondary.

# Tool tests

Each Design/Photo tool: activation, cursor, hover target, preview, numeric takeover if applicable, modifiers, commit, cancel, tool switch during operation, undo, redo, save/reopen, invalid selection, accessibility and status hint.

# Panel tests

Layers, Properties, Color, Swatches, Stroke, Transform, Align, Typography, Paragraph, Text Styles, Pages/Surfaces, Assets, Symbols, Styles, History, Navigator, Data Merge, Export, Histogram, Adjustments, Channels, Brushes, Brush Settings, Masks, Resources/Links, Background Tasks and Plugins according to scope.

# First-run proof

Clean preferences/cache. Launch. Create/open document. Use Design tools. Switch Photo without conversion. Modify property. Undo/redo. Save .PTND. Close. Reopen. Export. No hidden setup or developer state.

# Accessibility proof

Keyboard: New/Open → select object/layer → Properties → edit value → save/export. Photo: layer → adjustment → edit → save. Screen-reader semantics where platform/Slint support exists. IME composition does not trigger global shortcuts.

# Localization proof

en-US, pt-BR, German expansion, pseudo-locale, Japanese IME, Arabic/RTL sample. No clipped primary actions or off-screen menu labels.

# Performance proof

Measure UI frame/p95 or equivalent interaction traces for Layers scroll, docking drag, typing, command palette, panel resize, canvas interaction and large list models. Record hardware/build/environment. No invented numbers.

# Visual proof

Golden/component gallery plus representative full-shell scenes: Design default, Photo default, light theme, compact density, floating panel, Export, command palette, Preferences, error stack. Never auto-accept baseline simply to make diff pass.

# Final Slint gate

Failed required=0; untested required=0; dead controls=0; placeholder surfaces=0; stale critical visual/interaction evidence=0; forbidden Slint dependency edges=0; unresolved high-impact keyboard/a11y defects=0; known unwaived UI regressions=0.