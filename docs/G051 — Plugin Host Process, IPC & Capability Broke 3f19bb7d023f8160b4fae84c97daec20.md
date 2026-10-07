# G051 — Plugin Host Process, IPC & Capability Broker

# Goal

Establish secure third-party Python plugin runtime.

# Depends

Process topology, 09.14, 24.1–24.3, security gates.

# Primary

architect + security-architect.

# Deliverables

petunia-plugin-host; authenticated IPC handshake; schema validation; process lifecycle; capability/file/network broker baseline; quotas/timeouts; crash isolation.

# Acceptance

Hostile sample plugin cannot access denied file/network/process/env; crash/timeout leaves main app/document intact.

# Tests

Protocol fuzz, permission matrix, sandbox adversarial suite, restart/shutdown.

# Non-goal

Rich plugin SDK/UI (G052).