# 01.1 — Layered Architecture, Dependency Direction & Composition Root

# Layers

1. Domain/Core C++: canonical data, invariants, commands, geometry-neutral contracts.
2. Application C++: sessions, actions, queries, jobs, orchestration facades.
3. Native infrastructure: IO, renderer, codecs, platform-neutral adapters.
4. Python application: workflow/tool/panel orchestration.
5. Qt presentation: widgets/models/actions/windows.
6. External adapters: plugins, MCP, OS integrations.

# Dependency direction

Dependencies point inward toward stable contracts. Domain knows nothing about Qt/Python/plugin transports. Presentation knows semantic facades, not storage internals.

# Composition root

apps/petunia_studio/bootstrap constructs concrete implementations and injects them into application services. Global accessors/service-locator prohibited.

# Boundary tests

CI validates target/import graph. A headless test shell must open/edit/save/export without Qt. A Qt shell must be replaceable without changing document schema.

# Cross-cutting

Logging, diagnostics, permissions and cancellation flow via explicit interfaces/context, not static globals.

# Rule

New subsystem must document owning layer, inbound/outbound dependencies and why any exception is safe.