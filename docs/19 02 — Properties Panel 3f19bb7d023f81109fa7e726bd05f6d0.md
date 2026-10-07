# 19.02 — Properties Panel

# Identity

PanelId [ptnd.panel.properties](http://ptnd.panel.properties).

# Purpose

Complete schema-driven inspector for current selection/tool/document context.

# Source

PropertySchemaRegistry + PropertyValueSnapshot. Common properties ordered by semantic sections: Transform, Geometry, Appearance summary, Type-specific, Layout, Metadata/Advanced.

# Multi-selection

Properties can be Same, Mixed, Unavailable or PartiallyApplicable. Editing a partially applicable property requires explicit compatible-target policy and feedback count.

# Editors

Numeric, enum, bool, color, resource, text, list, composite transform, curve/profile specialized editors. Generic renderer preferred; custom editors registered by schema control type.

# Edit lifecycle

BeginEdit token -> staged preview -> Commit/Cancel. Selection change during staged edit resolves safely. Continuous edits one history entry.

# Sections

Stable collapse state per object type/workspace; search can filter property labels/help/IDs.

# Reset/inheritance

Show whether value local, style-linked, inherited/default. Reset menu targets appropriate source.

# Errors

Inline validation with PropertyId/error code and no mutation. Disabled property shows reason.

# Plugin

Declarative plugin properties can appear via safe schemas; plugin process cannot inject QWidget.

# Accessibility

Labels-for, mixed state, units/ranges, section headings, keyboard traversal.

# Tests

Mixed values, selection change mid-edit, validation, plugin schema, undo, search and 100-property object.