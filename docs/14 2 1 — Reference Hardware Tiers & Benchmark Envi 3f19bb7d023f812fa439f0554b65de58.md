# 14.2.1 — Reference Hardware Tiers & Benchmark Environment

# Purpose

Performance claims are valid only against named hardware/environment. These are initial engineering target tiers and must be replaced/refined by actual lab machines before release.

# Tier L — Minimum target

4-core/8-thread contemporary x86-64 or ARM64 CPU, 8 GB RAM, integrated GPU with modern Vulkan/Metal/D3D capability, 2 GB effective graphics memory/shared budget, SSD, 1920×1080 60 Hz.

# Tier R — Recommended

8-core CPU, 16 GB RAM, midrange discrete/integrated modern GPU with >=4 GB practical graphics budget, NVMe SSD, 2560×1440 60–120 Hz.

# Tier H — Heavy professional

12+ core CPU, 32+ GB RAM, discrete GPU >=8 GB VRAM, NVMe, 4K/HiDPI dual display.

# OS matrix

Current supported Windows, mainstream Linux Wayland/X11 package target and current supported macOS versions defined by release policy. Benchmark records exact OS/kernel/driver.

# Environment record

BuildId, compiler, build profile, Python/Qt version, renderer/backend/device/driver, power mode, display/DPI, document fixture, cache warm/cold status.

# Thermal/noise

Laptop runs record plugged/battery/performance mode. Repeated tests detect throttling; unstable runs discarded with reason.

# Release

Minimum spec marketing can only be declared after Tier L real-machine validation.