# 09.15.2 — MCP Transport, Authentication, Scopes, File Grants, Privacy & Audit

# Transport profiles

Local developer/client: stdio or local socket.

Remote/team automation: Streamable HTTP compatible transport behind authenticated server.

Transport is adapter; domain permission checks identical.

# Client identity

Connection has ClientId, display metadata, granted scopes and session. Remote auth uses approved OAuth/token mechanism; secrets stored credential service.

# Scopes

[app.read](http://app.read), [document.read](http://document.read), document.write, [file.read](http://file.read)_granted, file.write_granted, export, import, clipboard, [settings.read/write](http://settings.read/write), [plugin.read/admin](http://plugin.read/admin), ui.inspect, developer.screenshot, developer.synthetic_input, diagnostics.

# Consent

First connection or scope escalation shows clear UI when interactive policy requires. Admin/headless deployments configure explicit policy. “[localhost](http://localhost)” does not imply all scopes.

# File grant

Open/save picker creates opaque GrantId with operation/read/write, paths or portal handles, expiry/persistence and conflict policy. MCP methods use GrantId, not arbitrary path, unless explicit trusted policy enables path mode.

# Privacy

document.summary avoids artwork text/pixels by default. Object detail returns requested semantic data; binary/pixel extraction is separate high-sensitivity method/scope if ever provided.

# Rate limits

Per-client request/message/job limits. Expensive queries return pagination/job. Misbehaving client cannot stall Qt thread.

# Audit

Connection, scope grant/revoke, ActionId mutation, file operation target classification, job and result. Redact secrets/document content.

# Revocation

MCP Manager can terminate client/session and revoke tokens/grants. In-flight mutation transaction either completes atomically or cancels/rolls back according stage.