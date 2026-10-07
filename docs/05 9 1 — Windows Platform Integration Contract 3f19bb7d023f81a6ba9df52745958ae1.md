# 05.9.1 — Windows Platform Integration Contract

# Window/input

Qt Windows platform plugin baseline; test Win11 supported releases, per-monitor DPI awareness, pen/Windows Ink behavior and IME.

# Files

Native dialogs, long paths/Unicode, atomic replace semantics, file associations .PTND, Recent Docs optional and Explorer Reveal.

# Color

Monitor ICC retrieval/change events through platform adapter; HDR display is research until renderer/color pipeline explicitly supports it.

# Packaging

Installer/MSIX choice via release ADR, code signing, uninstaller, VC runtime/static policy and GPU driver diagnostics.

# Accessibility

UI Automation bridge through Qt/QAccessible verified with NVDA/Windows tools.

# Security

Use known folders/temp APIs, no shell string commands, credential store via Windows Credential Manager where needed.

# Tests

150/200% mixed DPI, pen, IME, locked file save, OneDrive-like path behavior and signed installer smoke.