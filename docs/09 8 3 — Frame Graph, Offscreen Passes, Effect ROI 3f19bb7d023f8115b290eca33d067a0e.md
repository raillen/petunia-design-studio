# 09.8.3 — Frame Graph, Offscreen Passes, Effect ROI, Transient Textures & Scheduling

# Planner

RenderPlanner converts RenderScene into backend-neutral FrameGraph passes and resource dependencies.

# Pass classes

MainSurfacePass, OffscreenGroupPass, MaskPass, EffectPass, ColorTransformPass, OverlayPass, Resolve/Present.

# Offscreen requirement

Isolation, live effects, complex masks and non-local blend semantics can allocate intermediate target. Planner minimizes offscreen usage without changing output semantics.

# ROI

Each effect declares:

- input ROI;
- output ROI expansion;
- sampling neighborhood;
- tileability;
- quality levels.

Gaussian blur expansion for example derives from sigma/radius rule defined in effect contract.

# Damage

Canonical/view ChangeSet -> conservative damage bounds -> planner restricts redraw where all participating effects/groups support partial evaluation. If uncertain, expand/full redraw rather than produce seams.

# Transient allocator

FrameGraph computes lifetimes and reuses compatible temporary textures after last use. Transient resources never outlive frame/fence incorrectly.

# Synchronization

Backend owns barriers/fences. FrameGraph expresses read/write dependencies; no ad-hoc GPU synchronization in effect implementations.

# Async upload

Images/glyphs/vector buffers can upload before use; frame either waits within budget or uses known fallback/stale-safe resource according policy, never uninitialized memory.

# Quality

Interactive passes can select preview resolution/iterations. Final/idle redraw schedules full-quality version and replaces atomically.

# Tests

Nested masks/effects, overlapping offscreen lifetimes, partial redraw under blur, cancellation/resize, deterministic pass plan snapshot and resource lifetime validation.