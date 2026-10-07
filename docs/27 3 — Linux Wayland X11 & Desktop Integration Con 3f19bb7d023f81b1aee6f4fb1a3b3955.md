# 27.3 — Linux Wayland/X11 & Desktop Integration Contract

# Display

Wayland is first-class target; X11 supported through Qt where practical. Renderer surface creation abstracts window-system differences.

# Portals

xdg-desktop-portal for file chooser/open URI and sandbox-friendly integration where packaging/environment requires.

# GPU

Vulkan/OpenGL/fallback support depends renderer choice and driver matrix. Mesa/NVIDIA/AMD/Intel representative systems included in verification.

# Input

libinput/Qt tablet/stylus paths vary by compositor; pressure/tilt/button behavior tested on representative Wayland compositors and X11.

# Color

Linux color-management ecosystem varies. Adapter discovers ICC via supported desktop/colord/portal mechanisms where available; absence yields explicit unmanaged/system fallback status.

# Packaging

Flatpak candidate for sandboxed mainstream distribution; AppImage/native packages may supplement. Bundled Qt/Python avoids distro Python mismatch.

# Accessibility

AT-SPI/Orca through Qt accessibility. Keyboard-only operation cannot depend on desktop-specific global menu.

# Desktop matrix

GNOME and KDE primary; other desktops best-effort once Qt behavior validated.

# Tests

Wayland scaling/fractional scaling, X11, portals, file grants, clipboard/DnD, pen, IME and theme independence.