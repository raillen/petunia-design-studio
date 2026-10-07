# 04.2 — PySide6 / Qt 6.12 LTS Versioning, Licensing, Modules & Deployment Profile

# Baseline

**Qt 6.12.0 LTS / PySide6 6.12.x** is the canonical GUI baseline as of 2026-10-06. Qt 6.12 was released 2026-09-30 and is an LTS line.

# Primary Qt modules

QtCore — events, models, settings adapters, timers, threading bridges.

QtGui — actions, input, fonts, images, clipboard, accessibility primitives, window/surface integration.

QtWidgets — primary professional desktop shell and controls.

QtNetwork — only through controlled application services where needed.

QtSvg — UI icon/vector asset rendering when appropriate.

QtOpenGL/QRhi-related integration — only as renderer hosting adapter if selected by ADR, never document renderer authority.

QtTest — component/UI testing.

# Avoid unnecessary modules

QtQuick/QML is not a baseline dependency for the main shell. It may be evaluated for a specialized surface only through ADR and cannot create a second UI architecture casually.

QtWebEngine is excluded by default due size/security footprint unless a concrete embedded-web requirement appears.

# Licensing

PySide6 LGPL/commercial terms and all Qt module redistribution obligations must be reviewed by release/licensing gates. The design must not assume a commercial Qt license for core viability.

# Deployment

Bundle Qt plugins explicitly:

- platform plugin;
- image format plugins actually required;
- icon/theme integrations if used;
- TLS backend if network features ship.

CI/package smoke tests must catch missing plugin/runtime deployment.

# Styles

Do not depend on platform Qt style as design identity. Petunia owns tokens and custom delegates/widgets where needed, while respecting native window/menu conventions.

# Updates

Patch-level Qt/PySide updates enter via dependency PR + UI/platform smoke + regression suite; LTS does not mean blind auto-upgrade.