# 27.2 — macOS Platform Contract

# Minimum OS

Locked from Qt/renderer/toolchain support before release.

# Windowing

Cocoa-integrated Qt windows, global menu behavior, document tabs/windows, Retina DPR and Spaces/fullscreen conventions.

# GPU

Metal path directly or through chosen backend; device/layer lifecycle handles resize/screen moves and sleep/wake.

# Pen/input

Tablet/trackpad pressure/gesture capabilities through Qt/native adapter. Trackpad pinch/pan first-class.

# Color

ColorSync/display profile integration through adapter; per-monitor changes trigger color transform rebuild.

# Files

Security-scoped bookmarks if sandbox/notarization packaging uses them, Unicode normalization/path behavior and atomic replace on APFS tested.

# Packaging

Signed/notarized universal or architecture-specific app strategy, embedded Python/Qt/native dependencies and update signature.

# Accessibility

VoiceOver semantic/control workflows, keyboard Full Keyboard Access and high-contrast/reduce-motion system preferences.

# Tests

Retina/non-Retina monitor transitions, Apple Silicon/x86 policy, notarized clean install, font discovery and IME.