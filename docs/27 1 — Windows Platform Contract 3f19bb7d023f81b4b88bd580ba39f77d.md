# 27.1 — Windows Platform Contract

# Minimum OS

Lock after Qt 6.12/backend evidence. Installer blocks unsupported versions with clear requirement.

# Windowing

Qt Widgets/QWindow shell, native file dialogs where appropriate, multi-monitor DPI changes and taskbar/file-association integration.

# GPU

Production backend maps to D3D12/Vulkan/other selected API. Driver capability/denylist registry records vendor/device/version workaround.

# Pen

Windows Ink/tablet events normalized; pressure/tilt/rotation/eraser and high-frequency sample behavior tested.

# Color

Display profile discovery/monitor change adapter; HDR display support is separate post-V1 capability unless explicitly validated.

# Files

Long paths/Unicode, atomic replace, file watcher, removable/network volumes and Windows file locking tested.

# Packaging

Signed installer/package, bundled VC/runtime where required, Qt/Python/native libs, PTND association and updater.

# Accessibility

UI Automation via Qt accessibility plus NVDA/JAWS tests for critical workflows.

# Tests

100/125/150/200% scaling, mixed monitors, GPU reset, sleep/wake, pen, paths and install/upgrade/uninstall.