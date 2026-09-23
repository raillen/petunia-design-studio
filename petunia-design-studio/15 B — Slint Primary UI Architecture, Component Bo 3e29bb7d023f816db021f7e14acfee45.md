# 15.B — Slint Primary UI Architecture, Component Boundaries & Shell Contract

# Canonical UI architecture

**Slint Window/Components → Petunia Design System → Presentation Models/Semantic Components → PetuniaDesignGuiBridge → Application Ports/Actions/Commands/Jobs → Document/Engines/Persistence/Render Scene.**

# Absolute boundary

No Slint component, property, callback, model, color, brush, image type, window handle or event type may cross into canonical domain/application APIs. Slint is a shell adapter.

# Suggested Slint source topology

ui/app_window.slint; ui/tokens; ui/themes; ui/primitives; ui/components; ui/shell; ui/menus; ui/panels; ui/dialogs; ui/canvas; ui/design; ui/photo; ui/accessibility; ui/dev_gallery.

# Primitive components

PetuniaButton, IconButton, ToolButton, SegmentedControl, TextField, NumericField, SliderField, ComboBox, PopupMenu, MenuRow, Tooltip, Panel, PanelTab, PanelSection, TreeRow, VirtualList adapter, Splitter, DockTargetOverlay, ModalShell, Popover, Toast, StatusItem, ColorSwatch, ColorPickerShell, SearchField, TabStrip, PropertyRow, EmptyState, InlineDiagnostic and ProgressRow.

# Component contract

Every reusable component defines default/hover/pressed/focus/disabled/selected/mixed/loading/error states, pointer hit area, keyboard behavior, focus semantics, accessible label/state, density behavior, theme tokens, localization expansion and deterministic test ID.

# Models and performance

Layers, Assets, Fonts and History require incremental or virtualized models. Never rebuild a 10k-row tree because of hover or unrelated document changes. UI presentation models are derived and disposable; the document remains canonical elsewhere.

# Canvas host

The creative canvas must not own document semantics. Slint shell owns viewport geometry, focus, pointer/pen/key capture, DPI, cursor and overlay host lifecycle. Scene extraction/rendering remain engine responsibilities.

# Docking

Implement Petunia docking with left/right/bottom zones, tab stacks, splitters, drag targets, float/redock where supported, workspace persistence, unavailable-panel recovery, min/preferred sizes and keyboard move commands. A fixed-sidebar prototype cannot be called V1-complete.

# Menus and dialogs

Every menu item resolves a semantic ActionId. Native/system dialogs remain behind PlatformRequest adapters; rfd or equivalent can be used only inside platform/UI adapters.

# Slint accessibility

Use Slint accessibility semantics where available and preserve explicit Petunia role/name/value/state/action metadata. Missing toolkit support is a tracked gap, not a silent exception.

# Shell conformance

MockGuiAdapter and Slint adapter must pass the same action/property/selection/dialog/job/document-lifecycle semantics plus toolkit-specific focus, accessibility, DPI, rendering-host and input-transform tests.