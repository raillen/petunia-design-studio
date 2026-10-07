# 09.1 — Modularity, Capability Registry & Contribution Architecture

# Principle

Feature é contribution, não switch espalhado. Core capabilities são registradas por interfaces estáveis.

# Registries

ActionRegistry, ToolRegistry, PanelRegistry, PropertySchemaRegistry, ImporterRegistry, ExporterRegistry, EffectRegistry, DataSourceRegistry, ResourceProviderRegistry e HelpRegistry.

# Module contract

Cada module declara ModuleId, version, required capabilities, provided contributions, startup/shutdown hooks e diagnostics. Domain modules não importam UI modules.

# Detach proof

Module deve poder ser desabilitado em teste sem quebrar unrelated modules. Saved workspace/document references a unavailable optional capability geram degraded state, não crash.

# Built-in vs plugin

Built-ins usam mesmos semantic registries, mas podem ter trusted native implementation. Plugins recebem façades brokered.

# Dependency rule

Contributions dependem de contracts; contracts dependem de core value types. No lateral import cycles. CI roda dependency graph/cycle gate.

# Lifecycle

register -> validate conflicts -> activate -> contribute -> suspend/disable -> unregister. Unregister nunca invalida canonical document sem explicit missing-capability representation.