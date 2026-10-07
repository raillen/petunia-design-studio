# 09.15.6 — MCP Tool Permissions, Remote Deployment, Audit & Abuse Resistance

# Scope mapping

Every method declares minimum scope. Generic actions.execute additionally intersects ActionDescriptor permissions/capabilities.

# Remote

Remote Streamable HTTP requires authentication, TLS termination policy and explicit enablement. Default desktop install exposes no unauthenticated LAN endpoint.

# Session

Auth principal creates MCP session with granted scopes and optional document restrictions. Idle/absolute expiry configurable.

# File

All ordinary file methods consume GrantId returned from user/admin policy, not arbitrary path. Trusted headless profile can explicitly enable path mode.

# Abuse controls

Rate limits, concurrent job limits, query result caps, payload caps and CPU/time budgets. Repeated invalid requests can throttle/disconnect.

# Audit

Record client, method/ActionId, object counts, file grant identity classification, revision before/after, duration/result. Redact text/pixels/secrets.

# Dangerous scopes

plugin.admin, settings.write, [clipboard.read](http://clipboard.read), diagnostics sensitive, developer synthetic input/screenshot require explicit elevated grant and may be unavailable remotely.

# Tests

Unauthorized method matrix, scope revocation mid-session, rate exhaustion, remote disabled by default, audit redaction and disconnect cleanup.