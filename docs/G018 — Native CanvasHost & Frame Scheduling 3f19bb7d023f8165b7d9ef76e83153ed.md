# G018 — Native CanvasHost & Frame Scheduling

# Goal

Embed a native render surface in Qt with correct lifecycle/input/frame scheduling before real renderer breadth.

# Depends

G014, G008.

# Primary

renderer-engineer + ui-component-engineer.

# Skills

rendering-2d, lang-cpp, lang-python, ui-implementation.

# Deliverables

CanvasHost contract; QWidget/QWindow adapter candidate; native window/surface handle extraction; DPR/resize/expose events; ViewportController; dirty/requestFrame coalescing; placeholder clear/test frame; normalized pointer/keyboard event forwarding; pointer capture.

# Acceptance

Canvas survives resize, minimize, DPI move and multiple tabs/windows without leaks/crash. Idle does not render continuously. Synthetic input reaches semantic event log.

# Tests

Resize storm, hidden window, mixed DPI, capture lost, two canvases, shutdown and native resource cleanup.

# Non-goals

Final GPU backend, vector rendering.