# 14.3.3 — MCP Authentication, Authorization, Replay, Grant & Abuse Security Tests

# Authentication

Missing/invalid/expired credential, wrong principal/client, remote TLS/proxy policy and session fixation attempts.

# Authorization

Every method exercised with no scope, minimum scope, neighboring insufficient scope and admin scope. Document read-only/lock still enforced after auth.

# Replay/idempotency

Repeat mutation request with same/new requestId/idempotency key; simulate response loss; stale expectedRevision. No duplicate mutation.

# File grants

Path substitution, traversal, symlink/reparse, expired grant, read-vs-write escalation and grant to one file then access sibling.

# Abuse

Huge pagination limit, rapid jobs, subscription flood, deeply nested JSON, many connections, expensive query filters and cancel storms.

# Audit

Verify sensitive payload redaction while preserving principal/client/method/outcome/correlation.

# Remote

No developer synthetic-input/screenshot/admin methods unless explicit scopes/policy; [localhost](http://localhost) does not skip tests.