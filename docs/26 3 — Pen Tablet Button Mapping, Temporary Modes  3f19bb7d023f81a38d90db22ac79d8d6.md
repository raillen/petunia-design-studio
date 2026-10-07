# 26.3 — Pen/Tablet Button Mapping, Temporary Modes & Device Profiles

# Device abstraction

Input system identifies stylus buttons/eraser/pressure/tilt/rotation independent from vendor driver naming.

# Defaults

Tip = primary action.

Eraser end = temporary Eraser operator/tool when supported.

Barrel button 1 candidate = temporary Pan or context menu by preference.

Barrel button 2 candidate = secondary/eyedropper/modifier according device/profile.

# Mapping

User can map device button to ActionId, temporary ToolId or modifier virtual key. Mapping is per device class or specific device ID if stable.

# Safety

Button mapping cannot invoke destructive action without ordinary confirmation/availability. Lost pen proximity releases temporary modes.

# Pressure calibration

Per-device curve/deadzone can normalize pressure before tool dynamics; raw value retained in diagnostics.

# Tests

Wacom-like/Windows Ink/macOS tablet events, eraser flip, button chord, proximity loss and profile fallback.