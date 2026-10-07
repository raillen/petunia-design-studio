# 05.9.2 — macOS Platform Integration Contract

# Window/menu

Native macOS menu conventions, app menu/Preferences/Quit mapping, Retina scaling, trackpad gestures and tablet support.

# Files

NSOpenPanel/NSSavePanel via Qt/native, security-scoped bookmarks if sandbox/App Store profile ever used, Finder Reveal and document open events.

# Color

ColorSync/display ICC adapter; monitor move and profile changes.

# Packaging

.app bundle, hardened runtime, code signing, notarization, universal architecture policy and DMG/PKG distribution ADR.

# Accessibility

VoiceOver/AX API verification through Qt accessibility.

# Input/text

Command shortcuts, Option word movement, IME/CJK, dead keys and trackpad pinch/pan.

# Tests

Retina/non-Retina external monitor, Spaces/fullscreen, notarized launch, permission/bookmark persistence and VoiceOver.