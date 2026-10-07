# 19.20 — Plugin Manager / MCP Manager Dialog Specifications

# Plugin Manager

List plugin name/version/source/status; detail Overview/Permissions/Contributions/Diagnostics. Install package → verify → compatibility → permission consent → enable.

# Permission changes

Grant/revoke grouped scopes; escalation after update requires consent. Restart requirement shown. Crash/incompatible state offers restart/disable/remove.

# MCP Manager

Enabled transports, clients/sessions, scopes, connection origin, last activity, grants and audit summary. Revoke client/session/token and developer inspection toggles.

# Security UX

High-risk scopes use outcome language (“Can read files you choose”, “Can write active document”). [Localhost](http://Localhost) is not labeled inherently safe.

# Tests

plugin crash, permission revoke, incompatible update, remote MCP disabled, revoke active client and screen-reader labels.