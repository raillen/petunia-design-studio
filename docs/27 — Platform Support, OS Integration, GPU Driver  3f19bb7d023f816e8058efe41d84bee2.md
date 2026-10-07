# 27 — Platform Support, OS Integration, GPU/Driver Matrix & Deployment Contract

# First-class platforms

Windows, macOS and Linux are architectural targets. Exact minimum OS versions are locked before V1 packaging after Qt/backend compatibility evidence.

# Platform dimensions

Windowing, native menus/dialogs, file portals, clipboard/drag-drop, tablet/pen, color-profile discovery, fonts, HiDPI, GPU API/driver, sandbox/install/update and accessibility APIs.

# Linux

Wayland first-class with X11 compatibility where Qt/backend supports. xdg-desktop-portal used for sandboxed file operations. Flatpak/AppImage/package strategy decided by release ADR.

# Windows

Win32/modern Windows integration through Qt/platform adapters; D3D/Vulkan backend availability according renderer choice; installer/file associations/update verified.

# macOS

Cocoa/Metal integration through Qt/backend; signed/notarized app bundle; Retina/multi-monitor profiles and system accessibility.

# GPU

Renderer support matrix records vendor/driver/API, known workarounds and feature/fallback status. Device/driver denylist requires evidence and version ranges.

# CPU fallback

Core correctness/headless/export path remains available where GPU unavailable, with explicitly lower performance.

# Deployment

Application bundles compatible Python, PySide/Qt and native libraries; system Python is irrelevant.

[27.1 — Windows Platform Contract](27%201%20%E2%80%94%20Windows%20Platform%20Contract%203f19bb7d023f81b4b88bd580ba39f77d.md)

[27.2 — macOS Platform Contract](27%202%20%E2%80%94%20macOS%20Platform%20Contract%203f19bb7d023f816fb4ddf673541f0be2.md)

[27.3 — Linux Wayland/X11 & Desktop Integration Contract](27%203%20%E2%80%94%20Linux%20Wayland%20X11%20&%20Desktop%20Integration%20Con%203f19bb7d023f81b1aee6f4fb1a3b3955.md)

[27.4 — GPU/Driver Capability Registry, Fallbacks, Denylist & Diagnostics](27%204%20%E2%80%94%20GPU%20Driver%20Capability%20Registry,%20Fallbacks,%20%203f19bb7d023f81f08bf6d79d530acfab.md)

[27.5 — Installation, Updates, File Associations, Sandboxing & Offline Operation](27%205%20%E2%80%94%20Installation,%20Updates,%20File%20Associations,%20S%203f19bb7d023f819b9b86e518d88aff23.md)