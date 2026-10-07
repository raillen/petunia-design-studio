# 07.3 — AI / Code-Agent Development Policy, Grounding, Authority & Non-Hallucination Rules

# Role

Agents accelerate implementation but cannot create undocumented product semantics or silently revise architecture.

# Grounding order

Goal criteria -> canonical notebook/ADR -> schemas/public headers -> current implementation/tests -> exploratory evidence.

# Required behavior

Agent identifies affected boundaries, reads only relevant microcontext, cites/links authority in Goal dossier, distinguishes planned vs implemented, runs declared tests and records evidence.

# Forbidden

Inventing APIs/IDs, bypassing Actions for convenience, marking unrun tests as passed, broad refactors unrelated to Goal, copying reference-product proprietary UI/assets, changing security/performance contracts without ADR.

# Uncertainty

If implementation evidence contradicts docs, agent records drift and routes to architect/documentation-maintainer rather than silently choosing.

# Generated code

Clearly marked, reproducible and reviewed. Agent-written migrations/parsers/security-sensitive code require stronger independent review.

# Review independence

Implementation agent cannot be sole security/performance/release verifier on high-risk Goals.