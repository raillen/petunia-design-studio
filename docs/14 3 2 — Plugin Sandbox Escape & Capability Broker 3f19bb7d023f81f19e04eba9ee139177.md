# 14.3.2 — Plugin Sandbox Escape & Capability Broker Adversarial Test Matrix

# Adversary

Third-party Python plugin intentionally attempts operations outside granted capability.

# Cases

Read arbitrary home/config/env; write outside plugin storage/grant; spawn shell/process; direct outbound network; connect [localhost](http://localhost) services; clipboard without grant; inspect other plugin IPC/storage; access MCP credentials; exhaust CPU/RAM/messages/files; forge plugin identity/protocol.

# OS enforcement

Tests distinguish broker denial from OS sandbox denial. Where platform sandbox cannot block a class, residual risk and compensating process/user permission policy documented.

# Document

Plugin attempts direct raw object mutation, stale transaction, oversized batch and prohibited admin Action. Host must reject atomically.

# Failure

Malformed/hostile plugin messages cannot crash main app. Plugin process killed after repeated protocol violation/limit breach according policy.

# Evidence

Per-platform sandbox profile, denied syscall/access results, host diagnostics and unchanged document snapshot.