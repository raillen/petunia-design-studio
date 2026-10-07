# 05.7 — Property Editor Framework, Generic Inspector Widgets, Validation & Mixed Values

# Schema renderer

PropertySchema -> PropertyEditorFactory -> Qt control bound through PropertyBindingController.

# Control mapping

Boolean -> CheckBox/Toggle.

Enum -> Combo/Segmented.

Number -> NumericField/Slider combo.

Color -> ColorWell/editor.

Resource -> picker/reference field.

Transform/vector -> composite specialized editor.

Curve/profile -> dedicated editor with numeric alternative.

# Read model

PropertyValueState = Same(value), Mixed, Unavailable, Loading, Error. Widget presentation never invents a default for Mixed.

# Edit lifecycle

begin_edit -> staged preview -> update -> commit/cancel. Continuous controls share one transaction ID. Core performs final validation.

# Validation

Immediate local syntax/range hints; authoritative native validation returns structured code/path/message metadata.

# Revert

Default/style/inherited/original distinctions can expose reset/revert icon with tooltip describing target.

# Selection changes

Active editor safely finishes/cancels according control policy before rebinding. Never apply stale edit to new selection.

# Accessibility

Label association, units in accessible description, keyboard scrubbing alternative and mixed state semantics.