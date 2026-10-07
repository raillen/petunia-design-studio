# 25.1 — Reference Hardware Tiers & Benchmark Environment Contract

# Provisional tiers

Actual models are pinned in CI/benchmark inventory, not hardcoded forever.

# Low

4–6 core mainstream CPU, 16 GB RAM, integrated/entry GPU, 1080p/1440p.

# Recommended

8-core modern CPU, 32 GB RAM, mid-range discrete GPU with 8 GB VRAM, 1440p/4K.

# High

12+ core CPU, 64 GB RAM, high-end GPU 12+ GB VRAM, 4K/multi-monitor.

# Environment

Record OS/build, compositor/windowing (Wayland/X11), GPU/driver, display scale, power mode, app BuildId, backend, document fixture and cold/warm cache state.

# Benchmark discipline

No cross-machine comparison without normalization. Interactive latency runs with release/profile build and real UI/render path, not microbenchmark substitute.

# Update

Reference tier refresh is ADR/evidence change; retain historical results for trend.