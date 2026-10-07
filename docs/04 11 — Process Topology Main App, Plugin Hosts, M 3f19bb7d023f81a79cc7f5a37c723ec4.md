# 04.11 — Process Topology: Main App, Plugin Hosts, MCP Server, Helpers & Crash Containment

# Processes

**petunia-studio** — Python/Qt main process + in-process C++ core.

**petunia-plugin-host** — one/per-pool third-party Python plugin process.

**petunia-mcp** — may run embedded transport adapter or dedicated process depending deployment/security profile.

**codec/parser helper** — optional future isolation for risky formats.

**crash handler** — platform-appropriate minimal external collector candidate.

# Main-process authority

Only main process owns writable DocumentSession. Other processes send semantic requests validated into Actions/Commands.

# IPC

Versioned authenticated local IPC with schema limits. Never raw Python pickle, raw C++ pointers or shared QWidget.

# Shared memory

Allowed only for explicitly negotiated large immutable/transient buffers with ownership/lifetime protocol.

# Crash containment

Plugin/helper failure must not kill document process. Renderer device loss is handled in-process unless future GPU-process architecture is justified.

# Restart

Plugin helper restart re-registers contributions. MCP transport restart cannot mutate document without reconnect/auth/revision checks.

# Diagnostics

Each process carries BuildId and correlation IDs so one evidence bundle can reconstruct cross-process operation.