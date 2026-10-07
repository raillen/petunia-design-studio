# G22 — Minimal RenderScene & Rectangle/Image/Text Placeholder Primitives

# Goal

Implement first backend-neutral RenderScene pipeline on selected renderer.

# Depends

G21, G18.

# Authority

09.8.1–09.8.5.

# Owner

renderer-engineer.

# Deliverables

RenderFrameRequest, RenderScene/display-list representation, DrawPath rectangle primitive, DrawImage placeholder, DrawGlyphRun placeholder, clip stack, CPU/reference path where feasible, backend resource handles.

# Acceptance

Scene serializable/debuggable; same scene renders deterministic golden; no DocumentStore pointer reaches backend; resize/device-loss smoke passes.

# Evidence

CPU/GPU image diff and frame trace.