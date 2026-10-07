# 14.3 — Security Verification, Fuzzing, Hostile Inputs & Sandbox Evidence

# Fuzz targets

PTND ZIP/manifest/document JSON, SVG parser adapter, image metadata/codecs wrappers where feasible, ICC loader, geometry/path operations, expressions, plugin RPC messages, MCP schema validators.

# Corpus

Valid minimal/maximal plus malformed/truncated/duplicate entries/path traversal/decompression bomb/deep nesting/extreme numbers/invalid Unicode.

# Sandbox tests

Third-party Python plugin attempts forbidden file, network, process, env, clipboard and document operations. Verify denied at broker/OS layer and clear error.

# MCP

Auth/scope matrix, stale revision, malformed params, path grants, idempotency, rate/backpressure, audit redaction.

# Native safety

ASan/UBSan/TSan, static analysis, hardened release flags. Crashes in parser/helper/plugin process must not mutate original document.

# Supply chain

Dependency lock, SBOM, vulnerability audit, artifact checksum/signature, plugin package provenance and license audit.

# Evidence

Security review report pins revision, tool versions, commands, corpus duration/results, unresolved accepted risk and responsible owner.

[14.3.1 — Parser & File-Format Fuzz Targets, Corpus Strategy & Resource Limits](14%203%201%20%E2%80%94%20Parser%20&%20File-Format%20Fuzz%20Targets,%20Corpus%203f19bb7d023f812196eef0dc2886993f.md)

[14.3.2 — Plugin Sandbox Escape & Capability Broker Adversarial Test Matrix](14%203%202%20%E2%80%94%20Plugin%20Sandbox%20Escape%20&%20Capability%20Broker%203f19bb7d023f81f19e04eba9ee139177.md)

[14.3.3 — MCP Authentication, Authorization, Replay, Grant & Abuse Security Tests](14%203%203%20%E2%80%94%20MCP%20Authentication,%20Authorization,%20Replay%203f19bb7d023f811582cff51582b8d46c.md)

[14.3.4 — Native Memory Safety, Race Detection, Hardening & Crash Triage Protocol](14%203%204%20%E2%80%94%20Native%20Memory%20Safety,%20Race%20Detection,%20Har%203f19bb7d023f81c5ae03f31ce1723ee3.md)