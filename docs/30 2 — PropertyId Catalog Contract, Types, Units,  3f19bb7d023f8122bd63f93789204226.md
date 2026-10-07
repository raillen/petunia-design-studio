# 30.2 — PropertyId Catalog Contract, Types, Units, Mixed Values & Binding

# PropertyDescriptor

PropertyId, owner/type applicability, value type, unit/dimension, nullable/mixed, default/inherited semantics, validation/range, read/write, multi-edit policy, bindable, animatable future flag, UI control hint, permissions and serialization mapping.

# Names

[ptnd.property](http://ptnd.property).transform.x, [ptnd.property](http://ptnd.property).shape.rectangle.corner_radius, [ptnd.property](http://ptnd.property).stroke.width, [ptnd.property](http://ptnd.property).text.font_size.

# Values

Strong semantic types: Length, Angle, Percentage, ColorValue, Enum, Bool, String, ResourceRef, Transform, Curve, structured typed values. Avoid arbitrary JSON for core properties.

# Multi-edit

Descriptor declares ApplyAllCompatible, RequireSameType, SelectionAggregate or ReadOnlyAggregate. Mixed is a value-state, not null.

# Units

Canonical storage unit vs display unit defined. Numeric parser converts expressions/unit suffixes before command.

# Binding

Data Merge can bind only descriptors marked bindable and type-compatible; plugin/MCP discover same descriptor.

# Version

PropertyId remains stable across UI moves. If semantic meaning changes, introduce new ID/migration.

# Tests

Registry uniqueness, type validation, mixed edit, unit conversion, binding and generic inspector rendering.