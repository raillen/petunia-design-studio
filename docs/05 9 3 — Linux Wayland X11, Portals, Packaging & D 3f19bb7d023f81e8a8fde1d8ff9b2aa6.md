# 05.9.3 — Linux Wayland/X11, Portals, Packaging & Desktop Integration Contract

# Priority

Linux is first-class. Wayland is primary modern target with X11/XWayland compatibility according Qt support.

# Files

xdg-desktop-portal file chooser when sandboxed/Flatpak or portal preferred; direct native dialogs otherwise. Grant/token model abstracts both.

# Input

Wayland pointer/tablet/IME behavior tested; X11 differences isolated Qt adapter. Pen pressure/tilt matrix across common drivers/tablets where feasible.

# Color

ICC/display profile support varies by compositor/protocol; adapter reports capability honestly. Color-managed display limitations are surfaced, not silently claimed.

# Packaging

Flatpak candidate for sandbox/integration plus AppImage/package builds via ADR. Bundle Qt dependencies reliably.

# Accessibility

AT-SPI/Orca testing. Keyboard navigation cannot depend on desktop-specific global menus.

# Tests

GNOME/KDE representative, fractional scaling, Wayland/X11, portal file grants, Flatpak filesystem restrictions and Orca.