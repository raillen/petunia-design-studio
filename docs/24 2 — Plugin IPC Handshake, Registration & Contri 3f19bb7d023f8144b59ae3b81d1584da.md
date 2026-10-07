# 24.2 — Plugin IPC Handshake, Registration & Contribution Protocol

# Handshake

Host -> plugin: Hello{hostVersion, protocolVersions, sdkVersions, grantedPermissions, locale, capabilities, limits}.

Plugin -> host: HelloAck{pluginId, version, selectedProtocol, selectedSdk, requestedOptionalCapabilities, contributionDigest}.

# Registration

RegisterContributions carries typed descriptors for Actions, Panels, Tools, Importers/Exporters, DataSources, resources/help. Host validates IDs, permissions, conflicts and schema versions before activation.

# Lifecycle

Activate -> Registered -> Running -> Suspended/PermissionChanged -> Shutdown. Protocol includes heartbeat/health optional and graceful shutdown deadline.

# Capability change

If user revokes permission, host sends PermissionsChanged; plugin must invalidate unavailable operation. Broker independently enforces denial even if plugin ignores message.

# Contribution updates

Dynamic update allowed only for declared mutable contribution metadata/model; ToolId/ActionId identity/version changes generally require re-registration/restart.

# Failure

Invalid descriptor, duplicate ID or unauthorized capability rejects contribution without compromising other plugin contributions where isolation permits.

# Tests

Version negotiation, permission subset, ID conflict, plugin crash during registration, stale descriptors and restart.