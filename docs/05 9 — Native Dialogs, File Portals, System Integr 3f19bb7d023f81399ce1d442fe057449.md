# 05.9 — Native Dialogs, File Portals, System Integration, URLs, Clipboard & Notifications

# File dialogs

Use native/platform dialogs through IFileDialog. Linux prefers xdg-desktop-portal where sandbox/package environment requires.

# Grants

Dialog result can include security-scoped bookmark/portal handle metadata via platform adapter; document stores resource identity separately from permission token.

# File manager

Reveal file through platform service. No shell command string construction.

# URLs

Open documentation/source links through validated URI launcher; plugin URLs constrained by manifest/source metadata.

# Clipboard

IClipboard wraps Qt/system clipboard and permission-sensitive operations.

# Notifications

In-app notifications for workflow events. OS notifications only for background completion when app/window state and user preference justify it.

# Recent documents

Platform recent-doc integration optional; Petunia maintains own typed recents with privacy/clear control.

# File associations

Installer registers .PTND. Double-click/open-with routes OS open event to ApplicationLifecycle.

# Single instance

Optional single-instance coordinator forwards open requests to running app; failure safely starts new instance according policy.

[05.9.1 — Windows Platform Integration Contract](05%209%201%20%E2%80%94%20Windows%20Platform%20Integration%20Contract%203f19bb7d023f81a6ba9df52745958ae1.md)

[05.9.2 — macOS Platform Integration Contract](05%209%202%20%E2%80%94%20macOS%20Platform%20Integration%20Contract%203f19bb7d023f81378971f3783b31b7f3.md)

[05.9.3 — Linux Wayland/X11, Portals, Packaging & Desktop Integration Contract](05%209%203%20%E2%80%94%20Linux%20Wayland%20X11,%20Portals,%20Packaging%20&%20D%203f19bb7d023f81e8a8fde1d8ff9b2aa6.md)

[05.9.4 — OS Compatibility Matrix, Minimum Versions, GPU Drivers & Fallback Policy](05%209%204%20%E2%80%94%20OS%20Compatibility%20Matrix,%20Minimum%20Versions%203f19bb7d023f818ea8b7e43ba3c8d2f8.md)