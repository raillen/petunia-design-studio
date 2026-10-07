# 09.14.6 — Declarative Plugin UI Schema: Panels, Forms, Lists, Actions & Accessibility

# Principle

Third-party process describes UI semantics; Petunia renders trusted native controls.

# Panel schema

PanelId, title TextResource, icon resource, preferred dock regions/size, multi-instance, sections/layout groups, data source endpoints, actions and empty/loading/error descriptors.

# Controls

Label, Text, Button/Action, Toggle, NumericField, Slider, EnumSelect, ColorField, ResourcePicker, List/Table/Tree, Progress, Section, Tabs limited safe set. No arbitrary HTML/QSS/QWidget.

# Binding

Control value binds plugin-local model path or host PropertyId/Action where allowed. Events include control ID, validated value and model revision.

# Lists

Paged rows with stable RowId, columns/cells/actions, selection state. Host owns virtualization.

# Styling

Only semantic emphasis roles (normal/muted/warning/danger/accent) and layout hints. Plugin cannot specify arbitrary font sizes/colors that break accessibility/theme.

# Accessibility

Every interactive control requires visible/accessible label or explicitly associated text. Host validates schema and rejects inaccessible definitions.

# Localization

Plugin resources provide locale table keyed TextId with fallback language; no raw code-generated labels required at runtime.

# Errors

Validation errors mapped to control path; plugin crash replaces panel with standard unavailable state retaining dock placement.

# Tests

Schema validation, theme/density, keyboard/focus, 10k-row table, missing translations, plugin restart and permission-bound control.