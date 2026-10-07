# 12.8 — Requirement Status, Scope Taxonomy, Backlog Semantics & Ambiguity Elimination

# Status taxonomy

V1_REQUIRED, V1_STRETCH, POST_V1_CANDIDATE, RESEARCH, DEFERRED_ADR, NOT_PLANNED, IMPLEMENTED_UNVERIFIED, VERIFIED.

# Requirement anatomy

Stable ID, statement, rationale, authority page, milestone, dependencies, acceptance evidence and exclusions.

# No ambiguous verbs

“Support PDF” must specify import/export, versions/features, editability, fidelity and validation. “Affinity-like” must map workflow/capability, not serve as acceptance criterion.

# Backlog

Backlog item references requirement IDs and architecture impact. It is not allowed to redefine product semantics silently.

# Scope change

Promotion/demotion records reason and updates coverage/parity ledgers. Agent encountering ambiguity chooses safest existing contract or raises decision; never invents major feature behavior.

# Implementation status

Documentation clearly separates target spec from proven implementation to prevent agents/users assuming planned behavior exists.