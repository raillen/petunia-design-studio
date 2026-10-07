# 07.2 — Threat Model: Documents, Codecs, Fonts, Plugins, MCP, Filesystem & Updates

# Assets at risk

User documents, filesystem, credentials, network identity, privacy, availability, update integrity and plugin trust.

# Adversaries

Malicious PTND/external file, malicious plugin, compromised MCP client, malicious remote resource, dependency/supply-chain compromise and accidental malformed content.

# Trust boundaries

Archive/parser, codec/font/ICC libraries, plugin process, MCP transport/auth, file grant broker, network broker, updater and diagnostics bundle.

# Controls

Bounded parsing, path canonicalization, no pickle/eval, process isolation, capability grants, auth/scopes, TLS remote, signed updates, SBOM, fuzz/sanitizers, secret redaction and atomic save.

# Abuse cases

ZIP bomb, path traversal, malformed font, image dimension bomb, plugin exfiltration, MCP arbitrary path write, permission escalation, symlink race, updater downgrade and diagnostic data leak.

# Review

Security Architect maintains model; affected boundary change requires update and Security Reviewer evidence.