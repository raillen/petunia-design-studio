# 09.16 — Platform Services, Filesystem, Security, Diagnostics & Crash Handling

# Platform ports

IFileDialog, IClipboard, IUriLauncher, IFontDiscovery, IDisplayProfileProvider, ITabletProvider, IFileWatcher, ICredentialStore, IProcessBroker.

# Filesystem

Canonicalization before policy decisions; safe temp dirs; atomic write helpers; no path traversal from packages/plugins. External resource grants store durable bookmark/portal token where OS supports, not assumed permanent absolute access.

# Hostile inputs

PTND ZIP, SVG, PDF, images, fonts, ICC and plugin packages have size/time/depth budgets. Parsing happens in safe adapter boundary; high-risk codecs may move to helper process based threat analysis.

# Secrets

Never serialize auth tokens into document/workspace. OS credential store for remote MCP/plugin service credentials.

# Logging

Structured event code, subsystem, severity, session/doc IDs hashed/redacted as needed, timing and error code. Document contents absent by default.

# Crash bundle

App/build/platform/backend, recent structured events, stack dumps/minidumps, job state and sanitized config. User can inspect privacy summary before sending.

# Recovery on crash

Autosave journal remains independent. Crash handler does minimal async-safe work; no complex save from corrupted process state.

# Observability

Tracy/perf/ETW/Instruments hooks behind builds; Python profiling for application UI; renderer GPU markers.

# Security update

Dependency/SBOM scans, signatures/checksums for releases, plugin package provenance policy and updater least privilege.