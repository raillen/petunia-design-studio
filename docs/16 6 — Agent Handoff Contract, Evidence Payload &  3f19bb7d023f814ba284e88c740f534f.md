# 16.6 — Agent Handoff Contract, Evidence Payload & Independent Verification Rules

# Handoff payload

Goal/Task ID, immutable revision, changed files/modules, behavior delivered, authority pages used, commands/tests executed with status, evidence artifact paths, skipped gates, known limitations, follow-up risks and next agent.

# No narrative-only handoff

Statements like “looks good” or “should work” are insufficient. Every correctness/performance/security claim points to reproducible evidence.

# Reviewer input

Reviewer receives locked criteria, diff/revision and evidence; not hidden implementation conversation needed to understand intended behavior.

# Independence

Security Reviewer does not author the security-sensitive implementation under review. Performance Agent independently reproduces significant speed claims. Release Verifier verifies built artifacts, not source assumptions.

# Failed handoff

If tests failed/skipped, state explicitly. Next agent may continue only when Goal allows partial state; no fake Done status.

# Context economy

Handoff summarizes results and links evidence; it does not dump full logs into every subsequent microcontext.