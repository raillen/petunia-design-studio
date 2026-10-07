# 07.4 — Plugin & MCP Product Governance, Permission Changes, Deprecation & Compatibility

# Governance

Plugin SDK and MCP API are public product contracts and evolve more conservatively than internal Python/C++ APIs.

# Versions

Package schema, plugin semantic SDK, plugin IPC, MCP protocol/method schema, declarative UI schema and Action/Property IDs version independently.

# Permission evolution

New capability or broader authority is never smuggled into an existing permission. Plugin update requesting expanded authority triggers explicit consent/admin policy.

# Deprecation

Machine-readable deprecated-since, replacement, removal target and compatibility window. Host can warn developer without breaking installed workflows immediately.

# Compatibility

Reference plugins and MCP clients are exercised against supported host versions. Breaking change requires major version/adapter/migration.

# Stable semantics

Cosmetic renaming must not churn ToolId/PanelId/ActionId/PropertyId.

# Emergency response

Security revocation can disable plugin/API capability outside ordinary deprecation window with clear diagnostic and release note.