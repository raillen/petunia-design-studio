# 01.1 — Clean Architecture Layers, Dependency Rule & Composition Root

# Layers

Domain/Core — canonical values, document invariants, pure semantic services.

Application — Actions, Commands orchestration, sessions, query services, jobs, import/export coordination.

Infrastructure — filesystem, codecs, color/text/render backends, package readers, platform implementations.

Presentation — Python/PySide Qt widgets/models/controllers.

External adapters — plugins, MCP, CLI.

# Dependency

Outer layers depend inward on contracts. Domain never imports application/presentation/infrastructure concrete APIs. Infrastructure implements ports defined at inward layer.

# Composition root

apps/petunia_studio bootstrap constructs concrete platform/render/io services and injects ApplicationCore/Python application services. No service locator reachable globally.

# Cross-cutting

Logging/diagnostics receive structured events through narrow interfaces; cannot become hidden dependency graph.

# Factories

Backend/platform factories only in composition/infrastructure. Domain object factories enforce invariants but do not resolve global services.

# Testability

Core/app can run headless using in-memory filesystem, fake clock if needed, software renderer/reference adapters and deterministic job executor.

# Enforcement

CMake links/import-linter plus architectural unit tests fail forbidden edges.