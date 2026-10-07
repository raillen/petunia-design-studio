# 07.2 — Security Threat Model: Documents, Plugins, MCP, Codecs, Filesystem & Supply Chain

# Trust boundaries

Untrusted PTND/external files, fonts, ICC, SVG/PDF/images; third-party plugins; MCP clients; network responses; clipboard/drag-drop; package/update artifacts.

# Assets

Canonical document state, user files, credentials/grants, plugin permissions, application integrity, release/update channel and privacy of artwork/text.

# Threat classes

Path traversal, archive bomb, parser memory corruption, integer overflow, malicious font/profile, arbitrary process/network/file access, plugin escape, MCP auth bypass, stale/replay mutation, data exfiltration, dependency compromise.

# Controls

Bounded parsers, safe archive rules, native sanitizers/fuzzing, out-of-process untrusted Python plugins, capability broker, explicit file grants, network allowlists, typed Actions/Commands, revision checks, signed releases, SBOM and dependency audit.

# Principle

[Localhost](http://Localhost) is not automatically trusted. File extension is not identity. Plugin popularity is not permission. UI consent never replaces enforcement.

# Security evidence

Threat model update, tests/fuzz corpus, permission matrix, static/dynamic scans, release artifact verification and unresolved risk ledger.