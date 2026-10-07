# G052 — Plugin SDK, Declarative UI, Plugin Manager & Reference Plugins

# Goal

Make plugin development practical on secure host.

# Depends

G051, panels/actions/property schemas.

# Primary

editor/ui engineer + security review.

# Deliverables

typed Python SDK/stubs; Action/Panel/Tool/DataSource/Exporter contributions; declarative UI renderer; Plugin Manager install/enable/update/permissions/logs; reference plugins and packaging CLI.

# Acceptance

Reference plugins install, request permissions, contribute UI/actions/tools, survive host restart and fail cleanly when permission revoked.

# Tests

SDK typecheck, contribution conflicts, panel accessibility, permission escalation update and compatibility versions.