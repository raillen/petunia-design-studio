# 27.4 — GPU/Driver Capability Registry, Fallbacks, Denylist & Diagnostics

# Adapter info

Renderer reports backend, API version, GPU vendor/device, driver version, feature limits, memory budget and active workarounds.

# Capability registry

Required: render targets, texture formats, blend support, compute/shader capability as backend needs. Optional capabilities produce optimized path but must have fallback.

# Denylist

Specific driver/device/version ranges can disable feature/backend only with reproducible issue evidence. Rules are versioned, documented and visible in diagnostics.

# Startup selection

Auto chooses best supported backend; Preferences can offer troubleshooting override with restart if needed. Safe software/reference fallback available for recovery/headless.

# Crash/device loss

Repeated GPU failure can offer safe-mode relaunch instead of crash loop.

# Diagnostics

About/Support shows active backend/device/driver/workarounds and copyable sanitized report.

# Tests

Capability spoof/mock, device loss, allocation failure, denylist match, backend override and software fallback.