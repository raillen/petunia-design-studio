# 09.27 — UI-Agnostic Core, Ports/Adapters, Presentation Models & Shell Conformance

# Core purity

Core services compile/test without Qt, Python interpreter, window server or GPU when using headless adapters.

# Ports

Filesystem, clock if needed for metadata, resource resolver, color profile provider input, font source abstractions, renderer/output and job progress sinks are interfaces at proper layer.

# Presentation models

Application layer exposes immutable snapshots/view models for Layers, History, jobs, resources and action availability. They contain semantic IDs/data, not QWidget.

# Qt adapters

QAbstractItemModel wraps presentation model; QAction wraps action metadata; CanvasHost wraps RenderSession. Adapters may cache UI indexes but can rebuild from stable IDs.

# Shell conformance

A hypothetical CLI/test shell can create rectangle, edit properties, save/export using same Actions/Commands. If feature only works through QWidget callback, architecture is incomplete.

# Headless

petunia-cli validates, converts/exports supported docs, runs preflight and scripted semantic workflows without display server where backend supports.