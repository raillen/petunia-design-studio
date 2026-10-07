# 09.21 — Architecture Governance, ADRs, Compatibility, Implementation Sequence & Definition of Done

# Governance

Architect owns architectural consistency; no single feature PR may silently introduce new global boundary.

# ADR required for

renderer backend winner, public plugin ABI, breaking PTND schema, core threading model, new canonical color/pixel storage policy, major text engine replacement, security trust boundary, GUI framework replacement.

# ADR format

Context, constraints, options, evidence, decision, invariants, consequences, migration/rollback, status/revisit trigger.

# Compatibility dimensions

PTND reader/writer; Python facade; plugin semantic SDK; MCP schemas; workspace/preferences; Action/Property IDs.

# Deprecation

Mark replacement, warning window, tooling/migration, removal version. Stable semantic IDs should not churn for cosmetic renames.

# Sequence

Kernel boundaries precede broad tools. Headless semantics precede GUI special casing. Security boundary precedes third-party extension marketplace.

# DoD

Contract + implementation + tests + evidence + docs + compatibility impact + performance/security/accessibility gates applicable.