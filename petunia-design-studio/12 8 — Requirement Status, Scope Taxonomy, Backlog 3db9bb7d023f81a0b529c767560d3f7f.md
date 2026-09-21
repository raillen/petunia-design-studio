# 12.8 — Requirement Status, Scope Taxonomy, Backlog Semantics & Ambiguity Elimination

# Purpose

Code agents must be able to distinguish a release requirement from a future idea without guessing. Words such as `future`, `later`, `planned`, `eventually`, `maybe` and `candidate` are not sufficient status by themselves.

# Canonical requirement states

- **V1 Required** — part of the accepted V1 product/architecture and release-blocking when its milestone is reached.
- **Milestone Required** — required for a named current milestone but not necessarily all of V1.
- **Post-V1 Candidate** — explicitly not required for V1; architecture may remain future-compatible without implementing it.
- **Research / Prior Art** — may inform design; not an implementation requirement.
- **Open ADR** — implementation may proceed only within the documented safe boundary; final choice requires stated evidence/trigger.
- **Deferred by Dependency** — desired but blocked by an external maturity/capability condition that is named and measurable.
- **Deprecated / Migration Only** — retained only for compatibility/migration and must not be used by new implementation.
- **Historical** — context only; never canonical implementation guidance.
- **Out of Scope** — intentionally excluded unless a new product decision/ADR changes scope.

# Writing rule

Implementation-grade docs should say, for example, `Pattern fill — Post-V1 Candidate` rather than `pattern fill future`. A future-compatible data model may be V1 Required even while the feature itself is Post-V1 Candidate; state that distinction explicitly.

# Open ADR requirements

An Open ADR must include current safe behavior, candidates, what can proceed before the decision, evidence/benchmark needed, owner, compatibility impact and measurable close/revisit trigger.

# Backlog semantics

A Post-V1 Candidate is not permission for an agent to implement it opportunistically while touching nearby code. Implementation requires an explicit task/milestone transition. However, accepted V1 architecture should avoid gratuitously preventing plausible future features when the extension cost is low.

# No speculative abstraction

Future-proofing means preserving clean extension points, not building unused generic frameworks. Do not add unused plugin types, schema variants or indirection solely because a future feature is imaginable.

# Audit rule

Milestone documentation audits search for ambiguous scope words and either attach a canonical status or rewrite the sentence. Stable pages should not contain unclassified product requirements expressed only as `later`/`future`/`planned`.

# Agent behavior

When encountering an unclassified future statement, the agent treats it as **not authorized for implementation** and reports the ambiguity rather than silently promoting it to V1.

# Machine-readable status metadata

Where repository/VitePress documents describe scoped requirements, mirror the human status in predictable frontmatter or generated metadata (for example `status: v1-required`, `status: post-v1-candidate`, `status: open-adr`) so audits/agents can filter requirements without parsing prose heuristically. The canonical human meaning remains this taxonomy; metadata values must map one-to-one and CI validates unknown states.

# Status transition rule

Changing `Post-V1 Candidate`/`Research` to an implementation-authorizing status is a planning/product change, not an incidental code edit. Record who/what milestone promoted it, update the relevant Functional/Architecture/UI source pages first, then let Prumo/task dossiers and repository docs inherit the new state.