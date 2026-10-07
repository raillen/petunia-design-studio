# 09.14.5 — Plugin Capability & Permission Semantics: Exact Grants, Lifetimes & Revocation

# Capability model

Permission is a typed grant, not a boolean string only.

# Document grants

[document.read](http://document.read): selected/current/all-open scope options.

document.write: mutation through Actions/Transactions only; optionally restrict to active document.

[selection.read](http://selection.read): current view/session selection metadata.

No raw document memory.

# Filesystem

[filesystem.read/write](http://filesystem.read/write) uses GrantId with root/file, allowed operations, expiry/persistence, symlink/canonical-path policy and user origin. Plugin does not receive unrestricted home directory.

# Network

network.https allowlist host patterns, methods, redirect policy, byte/time/concurrency quotas. Credentials separately scoped and never exposed unless dedicated secret grant exists.

# Clipboard

read and write separated; read may require foreground/user gesture policy.

# UI

ui.panel, ui.action, ui.tool, ui.notification. Tool registration does not imply arbitrary canvas rendering or document write beyond granted Action path.

# Jobs/storage

[background.jobs](http://background.jobs) quota; [plugin.storage](http://plugin.storage) dedicated namespace with size limit.

# Admin

plugin.admin, settings.write and developer capabilities are high-risk and unavailable to ordinary plugins by default.

# Lifetime

Session-only, until app restart, persistent user grant or one-shot. High-risk permission escalation always explicit.

# Revocation

Broker checks on every privileged operation, not only startup. Revoked ongoing file/network stream terminates safely; document transaction either atomic complete or rollback.

# Tests

Permission matrix, revocation during job, nested path/symlink escape, redirect to forbidden host, clipboard denied and update requesting new scopes.