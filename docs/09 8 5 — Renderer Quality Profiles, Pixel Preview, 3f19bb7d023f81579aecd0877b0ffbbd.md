# 09.8.5 — Renderer Quality Profiles, Pixel Preview, Proofing, Device Loss & CPU Reference

# Quality profiles

InteractiveLow, InteractiveHigh, IdleHigh, ExportFinal, ReferenceCPU. Each effect/path/text/image operation declares how profile changes approximation, never semantics.

# Interactive

Permit lower-resolution blur/effect, coarser tessellation and deferred high-cost analysis while pointer is active. Must converge automatically after idle/commit.

# Pixel preview

Simulates final rasterization at target DPI/pixel grid and sampling policy. It is a view mode; document vector/text unchanged.

# Proof

Color proof transform and gamut overlay injected near final display stage. Export ignores monitor proof unless output preset explicitly asks for proof simulation.

# CPU reference

Headless deterministic compositor covers core drawing/effects needed for correctness oracle and CI. It need not equal GPU performance or support every accelerated specialty at first milestone, but unsupported reference gaps are tracked.

# Backend tolerance

Visual differential thresholds are operation/format-aware. Exact scalar operations can have tight tolerance; filter kernels may use bounded error metrics.

# Device loss

DeviceLost event cancels/abandons in-flight frames, disposes resources, recreates backend, republishes surface and redraws from newest coherent snapshot. No Document revision change.

# Backend fallback

Repeated initialization/device failure can offer software/reference fallback with diagnostics instead of data loss.

# Tests

Forced device loss, swapchain resize, monitor/DPI change, preview→idle convergence, CPU/GPU goldens and proof transform correctness.