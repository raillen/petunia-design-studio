# 13.6 — Documentation Depth Audit: Implementability Levels & Gap Taxonomy

# Purpose

Measure not only whether a topic exists, but whether an implementer can execute it without inventing missing semantics.

# Depth levels

L0 Mentioned — only feature/topic name.

L1 Intent — what/why and rough behavior.

L2 Contract — inputs/outputs/states/errors/ownership/persistence.

L3 Executable Specification — schemas, state machines, algorithms/invariants, Commands, edge cases, tests and budgets.

L4 Proven — implementation + deterministic evidence + conformance.

L5 Release Proven — packaged artifact evidence across supported platforms.

# Documentation target

Architecture boundaries, PTND, tools, panels, protocols, color/text/render/geometry/raster and security-sensitive areas require at least L3 before broad implementation.

Product roadmap/research pages can remain L1–L2 if explicitly marked non-implementation authority.

# Gap classes

Semantic Gap; Data/Schema Gap; State-Machine Gap; Numerical/Algorithm Gap; Error/Recovery Gap; Persistence Gap; Security Gap; Performance Gap; Accessibility Gap; Interop Gap; Evidence Gap.

# Audit row

Authority page, subject, current level, target level, gap classes, blocking milestone, owner/agent, next evidence.

# No false completion

A page with many paragraphs can still be L1 if it never freezes behavior. A short JSON schema/state table can be L3.

# Maintenance

Every milestone/release audit samples changed authority pages and downgrades status if implementation or evidence drift invalidates assumptions.