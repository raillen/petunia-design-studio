# 05.3 — Native Canvas Host, QWindow/QWidget Integration, Swapchain Lifecycle & Frame Scheduling

# Host choice

Canvas may be QWidget-backed native window or QWindow embedded according renderer integration spike. The public CanvasHost contract remains stable.

# Responsibilities

- native window/surface handle;
- size/DPR/exposure events;
- normalized input forwarding;
- frame request scheduling;
- cursor;
- accessibility overlay semantics;
- drag/drop focus boundary.

# Renderer ownership

C++ RenderSession owns swapchain/device-context resources for this view. Qt never owns canonical content rendering commands.

# Frame scheduling

ChangeSet/view change -> mark dirty -> request frame -> render newest coherent snapshot. Multiple invalidations coalesce. Do not blindly run permanent 60fps loop when idle.

# Resize

During resize, viewport updates immediately; render target recreation debounced/coordinated without stretched stale output longer than necessary.

# Occlusion

Hidden/minimized/unexposed canvas pauses expensive frames and background preview work.

# Device loss

Canvas reports surface lifecycle; RenderSession recovers backend and requests full redraw from scene snapshot.

# Screenshot/test

Offscreen RenderSession can produce deterministic image independent of window capture. UI screenshots are separate shell evidence.