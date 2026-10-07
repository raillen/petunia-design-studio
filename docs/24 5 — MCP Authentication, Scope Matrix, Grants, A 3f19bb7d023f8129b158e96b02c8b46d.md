# 24.5 — MCP Authentication, Scope Matrix, Grants, Audit & Remote Deployment

# Client/session

Each connection maps to ClientId + authenticated principal + granted scope set + active Petunia session bindings.

# Scopes

[app.read](http://app.read); [document.read](http://document.read); document.write; selection.write; export; import; [file.read](http://file.read)_granted; file.write_granted; clipboard; [settings.read/write](http://settings.read/write); [plugins.read/admin](http://plugins.read/admin); ui.inspect; developer.screenshot; developer.synthetic_input; diagnostics.

# Least privilege

Methods declare required scopes. Scope possession does not bypass document read-only state, object locks, format capability or Action validation.

# File grants

GrantId references user/admin-approved file/directory capability with allowed operations, canonical target/bookmark, lifetime and persistence. Remote clients do not send arbitrary server paths by default.

# Remote transport

TLS/authentication and reverse-proxy/deployment policy explicit. [Localhost](http://Localhost) is not treated as trusted omnipotence. CORS/browser exposure disabled unless separately designed.

# Audit

Connection/open/close, scope grant/revoke, mutation ActionId, file/export operations, plugin admin and developer input. Content payload redacted by default.

# Rate/backpressure

Per-client request/job/stream limits. Expensive query requires pagination/job.

# Tests

Unauthorized method, revoked scope mid-job, grant expiry, remote reconnect, audit redaction and client isolation.