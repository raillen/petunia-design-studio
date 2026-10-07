# 19.22 — About, Diagnostics, Job Center & Notification Specifications

# About

App version, BuildId, channel, Qt/Python/native versions, license links and copyright.

# Diagnostics

Renderer backend/device/driver, OS, display/DPI, cache/memory summary, plugin/MCP status and Copy Diagnostic. Redact usernames/private paths/tokens according policy.

# Job Center

List JobId/type/document/progress/phase/status; cancel/retry/reveal result actions. Finished successes auto-expire; failures persist until acknowledged/retried.

# Notifications

Success transient; warning actionable; error persistent when unresolved. Notifications reference Action/Job/Object and route to details without modal spam.

# Accessibility

Announcements rate-limited; progress semantics expose indeterminate vs percentage.

# Tests

redaction, many concurrent jobs, cancel, diagnostic copy and notification focus behavior.