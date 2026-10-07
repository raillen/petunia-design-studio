# 19.02 — Properties Panel Specification

# Identity

PanelId [ptnd.panel.properties](http://ptnd.panel.properties). Shared. Default right dock.

# Source

PropertySchemaRegistry + current selection/tool context. The panel is a generic renderer of typed property schemas, not a feature-specific mutation surface.

# Sections

Identity/transform, geometry, appearance, text/raster/effect-specific groups and extension/plugin groups ordered by schema metadata.

# Multi-selection

PropertyValueState = Same, Mixed, Unavailable, Loading, Error. Setting a value applies to eligible targets according schema's multi-edit policy.

# Edit lifecycle

Control focus begins edit session; live preview may update staged transaction; Enter/focus commit according control; Esc restores previous value. Continuous scrubbing coalesces one history item.

# Validation

Local syntax/range feedback plus authoritative core validation. Errors map to PropertyId and preserve rejected text until user corrects/cancels.

# Search

Optional property search for complex selections, matching label/help/PropertyId alias.

# Plugin properties

Declarative property groups inherit same host controls, permissions and accessibility.

# Accessibility

Every editor labeled with name, units, mixed state, current value, constraints and error. Section collapse state keyboard accessible.

# Tests

Mixed selection, selection change mid-edit, plugin property removal, validation error, undo grouping, 1k selected objects and schema evolution.