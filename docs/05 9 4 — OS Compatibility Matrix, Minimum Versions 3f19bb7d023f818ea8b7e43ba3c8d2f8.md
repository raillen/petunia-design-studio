# 05.9.4 — OS Compatibility Matrix, Minimum Versions, GPU Drivers & Fallback Policy

# Matrix fields

OS/version, architecture, Qt support, tested GPU/backend, driver floor, tablet, IME, color profile, accessibility, package type and status Supported/Best Effort/Unsupported.

# Minimum versions

Locked only after renderer/backend and Qt deployment evidence. Do not promise versions Qt/backend no longer supports.

# GPU

Maintain vendor classes Intel/AMD/NVIDIA/Apple plus software fallback. Known broken driver/backend combinations can auto-select safer backend with diagnostic.

# Startup probe

Collect renderer capability without invasive telemetry. If primary backend fails, try approved fallback and explain in diagnostics.

# Release

Stable release published only after smoke on supported matrix; untested configurations labeled best-effort.

# Docs

Support page generated from matrix/build pipeline where possible.