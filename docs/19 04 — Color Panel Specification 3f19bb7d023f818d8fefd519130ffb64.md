# 19.04 — Color Panel Specification

# Identity

PanelId ptnd.panel.color.

# Target

Always displays active color target: Fill, Stroke, gradient stop, text fill, effect color, brush color or sampled informational mode.

# Models

RGB, CMYK, Lab, Gray according document/target support. Spot/global swatch identity shown instead of flattening to numeric process values.

# Controls

Sliders/numeric fields, optional wheel/spectrum, alpha, profile/model selector, fill/stroke switch, swap/default/none and recent colors.

# Semantics

Changing display model can merely re-express same ColorValue; explicit Convert Color action changes canonical model/profile. UI distinguishes.

# Sampling

Eyedropper integration can preview sampled color before click/commit.

# Precision

Numeric formatting tied model; internal values retain full precision.

# Accessibility/tests

Slider labels include channel/model, numeric alternative always. Test profile change, global swatch link, spot tint, mixed selection and keyboard color entry.