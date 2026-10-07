# 09.25.1 — Property Type System, Units, Constraints & Mixed-Value Semantics

# Types

Bool, Int, Float, String, Enum, ColorValue, Transform, Vec2/Rect, ResourceRef, ObjectRef, Gradient, Curve/Profile, Array/Struct composite and read-only derived values.

# Descriptor

PropertyId, value type/schema, unit dimension, default/inherited source, min/max/step hints, validation, read/write, styleable, bindable, multi-selection policy and editor hint.

# Units

Length canonical numeric with UI conversion px/mm/cm/in/pt. Angle internal policy centralized. Percent normalized scalar. Unit dimension prevents incompatible expressions.

# Mixed

Query returns Same, Mixed, Unavailable, PartialCompatibility or Error. Set is atomic all-or-error unless descriptor explicitly allows compatible-only behavior and UI discloses it.

# Inheritance

Explicit, InheritedStyle, GlobalSwatchRef or Default. Reset/Clear Override/Detach Reference are distinct operations.

# Tests

Unit conversion, mixed set, style override, invalid enum/range and serialization mapping.