# 09.30 — Plugin Runtime ADR: Trusted Python, Out-of-Process Python & Optional WASM

# Decision

Python is the approachable primary plugin language because it matches application layer and ecosystem. **Untrusted third-party Python does not run in the main application process.**

# Tier A

Trusted built-in Python modules share interpreter/process, pass normal repository review and may use internal application APIs only when designated stable.

# Tier B

Third-party Python runs petunia-plugin-host process. Communication uses versioned RPC/serialization over local IPC; host receives only declared/brokered capabilities. OS sandboxing applied where platform permits.

# Tier C

Optional Wasmtime/WASI Component Model for stronger portable isolation and deterministic resource limits. It adapts same semantic SDK, not second business model.

# Rejected as V1 primary

Embedded Lua no longer necessary because application already owns Python. JavaScript adds another runtime without clear benefit for primary tier. Native C++ ABI too fragile/security-sensitive for public V1.

# Performance

Plugin process is not suitable for per-pixel/per-frame hot loops. Plugins request host effects/jobs or operate batches. Future high-performance extension uses approved WASM/native service capability, not raw GPU access.

# Failure

Plugin crash/timeouts terminate contribution and rollback uncommitted transaction; main app/document survive.

# Revisit

Only if real plugin workloads prove Python process latency/ecosystem inadequate or WASM maturity changes required capabilities.