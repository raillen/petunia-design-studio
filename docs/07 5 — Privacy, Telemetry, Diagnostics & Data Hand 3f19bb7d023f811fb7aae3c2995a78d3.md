# 07.5 — Privacy, Telemetry, Diagnostics & Data Handling Policy

# Default

Petunia professional local workflows do not require uploading documents or telemetry.

# Telemetry

If introduced, must be opt-in/clearly configurable, data-minimized, schema-documented and separable from crash reporting. No artwork pixels/text/file contents by default.

# Diagnostics

Structured technical events may include BuildId, subsystem, timings, error codes and redacted identifiers. Paths/usernames/secrets are sanitized.

# Crash reports

User can review summary; minidumps/logs avoid document payload where practical. Recovery files remain local unless user deliberately shares.

# Plugins/MCP

Audit logs record identity, scope, ActionId and outcome rather than content. Network plugin access is explicit permission.

# Support bundles

Bundle generator lists included files/categories and redacts credentials/tokens automatically.