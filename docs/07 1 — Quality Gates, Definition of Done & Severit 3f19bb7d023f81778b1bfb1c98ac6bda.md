# 07.1 — Quality Gates, Definition of Done & Severity Taxonomy

# Purpose

Define a single quality language across implementation, reviews, CI and release.

# Severity

P0 Critical — data loss, corruption, arbitrary code execution, security boundary bypass, catastrophic crash on common workflow.

P1 High — major workflow broken, severe fidelity error, persistent crash, inaccessible essential function, large performance regression.

P2 Medium — limited workflow defect, recoverable fidelity issue, secondary accessibility/UX problem, localized slowdown.

P3 Low — polish, minor inconsistency, low-risk edge case.

# Definition of Done

A feature is complete only when contract, code, deterministic tests, undo/save/load if applicable, UI/keyboard paths, plugin/MCP exposure decision, performance evidence, accessibility, docs and review are satisfied.

# Quality gates

Static, unit/property, integration, sanitizer, fuzz, visual/semantic, accessibility, performance, security and packaging gates are selected by impact matrix.

# Waiver

Any skipped gate needs reason, owner, expiration/revisit trigger and risk. “Not enough time” alone is not a permanent waiver.

# Regression rule

Every verified defect receives the smallest stable regression fixture/test practical.

# Release blocking

P0 always blocks. P1 blocks unless explicit release authority accepts documented risk. P2/P3 follow milestone policy.