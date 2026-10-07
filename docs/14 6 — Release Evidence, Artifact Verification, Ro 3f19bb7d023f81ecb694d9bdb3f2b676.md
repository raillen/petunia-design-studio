# 14.6 — Release Evidence, Artifact Verification, Rollback & Support Diagnostics

# Candidate

Immutable commit/build ID and dependency locks.

# Required evidence

Build matrix, unit/integration, type/lint, sanitizers, fuzz summary, performance comparison, UI smoke, format migration, plugin/MCP security, docs and licenses.

# Artifacts

Inventory installers/packages/CLI/debug symbols/schema/spec bundles with sizes and checksums/signatures.

# Installation

Clean install, upgrade from supported previous version, uninstall/reinstall, file association, first launch, plugin host/MCP defaults and offline launch.

# Rollback

Document compatibility statement: whether files saved by new version open in previous. Application rollback instructions and migration risk explicit.

# Diagnostics

Verify crash/log bundle redaction and About version/backend info. Support can map BuildId to source/artifacts.

# Verdict

Release Verifier outputs Go/No-Go with blockers and accepted waivers. Publishing is separate authorized action.