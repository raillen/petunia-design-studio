# 30.1 — ActionId Catalog Contract, Availability, Parameters & Side Effects

# ActionDescriptor

ActionId; owner module; TextId/title/help; category/tags/aliases; default shortcuts; parameter schema; availability function; check state; side-effect class; permissions; undo/revision semantics; since/deprecation.

# Side-effect classes

ViewOnly, SessionState, DocumentMutation, FileRead, FileWrite, Network, PluginAdmin, AppSetting, LongJob.

# Availability

Given ActionContext snapshot returns Enabled, Disabled(reason code), Hidden only when disclosure/security requires. Availability must not mutate or perform expensive blocking IO.

# Execute

Action resolves typed parameters/context into Command/Transaction or application service job. QWidget callbacks never bypass descriptor/dispatcher.

# Naming

ptnd.action.<domain>.<verb>. Examples ptnd.action.history.undo, ptnd.action.object.delete, ptnd.action.export.start.

# Discoverability

Command palette/menu/toolbars/plugin/MCP can query descriptor filtered by permission/context.

# Version

Changing parameter semantics incompatibly requires new schema/action version strategy; cosmetic label change does not change ID.

# Tests

Every built-in ActionId unique, help/TextId exists, declared permissions match side effects, UI/shortcut execute same path.