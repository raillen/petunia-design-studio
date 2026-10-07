# 12.3 — Design Patterns, Clean Architecture & Anti-Overengineering Rules

# Pattern principle

Use patterns to protect a real boundary/invariant, never because the pattern exists.

# Command

Canonical user/document mutation. Enables undo, replay, MCP/plugin parity and tests.

# Transaction / Unit of Work

Groups commands atomically; one logical gesture/task one history entry. Not a database abstraction.

# Adapter

Qt/platform, codecs, file formats, renderer backends, plugin transports and MCP transports.

# Strategy

Swappable algorithm/backend under stable semantics: boolean engine, raster resampler, renderer backend, color engine.

# Registry

Discoverable contributions: Action, Tool, Panel, Importer/Exporter, Effect, DataSource. Registry owns IDs/conflict validation, not business logic.

# State Machine

Interactive tools and lifecycle flows. Explicit states/events beat booleans such as is_dragging/is_editing combinations.

# Facade

Coarse-grained nanobind API and plugin SDK. Facade hides internal topology and reduces call overhead.

# Observer/Event Bus

Typed post-commit events/ChangeSets. Never use event bus to perform hidden document mutations.

# Model/View

Qt trees/tables; large datasets use QAbstractItemModel. Domain model is not a Qt item model.

# Builder

Complex immutable request objects like ExportRequest/ImportOptions can use typed builder, but only where constructor would be error-prone.

# Factory

Platform/backends selected in composition root. Avoid abstract factory towers with no alternative.

# Repository

Only for actual persistence/source boundary. Do not wrap every container in Repository.

# Visitor

Use sparingly for stable algebraic object hierarchies; variant/pattern dispatch may be clearer.

# Anti-patterns

God AppManager, global ServiceLocator, singleton mutable document, QObject domain entities, Controller that knows all panels, inheritance per tool merely to share two methods, generalized plugin API before actual contributions, premature custom allocator, raw JSON buses, event-driven hidden command chains.

# Clean code

Name by domain. Functions have one coherent reason to change. Modules own one capability. Prefer explicit duplicate small code over wrong abstraction; refactor after semantics stabilize.