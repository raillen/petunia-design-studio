# 18.16 — Eyedropper / Color Picker Tool Specification

# Modes

Document Source, Composited Document, Display Sample and Average Radius. Photo/Design share semantic picker with Persona-specific defaults.

# Source sample

Reads canonical object/color beneath pointer where semantically available; can return ColorValue/Swatch reference without display transform.

# Composited

Samples evaluated artwork before monitor transform in document working/compositor space then converts to chosen UI model.

# Display

Samples final displayed pixel after proof/display transform; clearly labeled because value may not equal document source.

# Average

Circular/square sample radius in device/document pixels per mode; async/coalesced for large radius if needed.

# Apply

Click can set active fill/stroke/color target only when “apply sample” mode enabled; otherwise informational. Modifier may copy full appearance through separate action.

# HUD

Shows model/profile/channel values and source mode.

# Tests

ICC proof mode, transparent edge, spot source, average radius and source-vs-display distinction.