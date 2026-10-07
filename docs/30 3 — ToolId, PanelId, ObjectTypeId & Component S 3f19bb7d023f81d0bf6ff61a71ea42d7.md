# 30.3 — ToolId, PanelId, ObjectTypeId & Component Semantic ID Rules

# ToolId

ptnd.tool.* descriptor links Persona, accepted targets/capabilities, cursor, context schema, activation ActionId and ToolSpec authority.

# PanelId

ptnd.panel.* descriptor links provider, default dock, personas, multi-instance, model type, help and permission. Workspace persists PanelId + instance ID.

# ObjectTypeId

ptnd.object.* tagged semantic type with schema/capabilities and migration. External plugins use their namespace.

# Component semantic IDs

UI test/accessibility/MCP inspection IDs identify stable logical controls such as context.stroke.width, not QWidget object addresses. Repeated rows use stable local object IDs.

# Collision

Built-in ptnd namespace reserved. Plugin reverse-domain IDs validated globally within registry and cannot shadow built-ins.

# Missing provider

Unknown optional Tool/Panel/Object extension remains placeholder/degraded representation according capability contract.

# Tests

Registry collision, workspace missing panel restore, plugin unload/reload and semantic UI ID uniqueness.