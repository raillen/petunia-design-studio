# 05.4 — Unified Input System: Mouse, Tablet, Touch, Gestures, Keyboard & Pointer Capture

# Normalized events

Qt events convert immediately to Petunia PointerEvent/PenEvent/GestureEvent/KeyEvent with timestamp, device ID/type, buttons, modifiers, local/document positions and tablet axes.

# Mouse

Move/down/up/double click/wheel. High-resolution wheel deltas retained for trackpad/precision zoom.

# Tablet

Pressure normalized but raw range/calibration metadata retained if useful. Tilt x/y, rotation, tangential pressure, eraser, barrel buttons. Device-specific quirks stay adapter-side.

# Touch

Touch points support pan/pinch; painting by touch disabled by default or user-configurable. Palm rejection leverages platform/Qt signals plus tool policy.

# Gestures

Pinch zoom centered on gesture centroid; pan; optional rotate canvas view if product enables it. Gesture recognizers cannot conflict silently with active pen stroke.

# Pointer capture

Tool gesture requests capture. Qt/window deactivation/lost capture produces explicit ToolEvent::CaptureLost so state machine can cancel/finalize safely.

# Keyboard

KeyEvent uses physical/logical representation sufficient for shortcuts/text distinction. Text input/IME remains TextInputService, not shortcut parser.

# Latency

Input queues coalesce hover/move where safe but never discard pen pressure/time samples required for brush quality.