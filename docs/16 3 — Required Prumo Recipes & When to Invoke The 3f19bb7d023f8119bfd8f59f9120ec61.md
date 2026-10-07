# 16.3 — Required Prumo Recipes & When to Invoke Them

# project-bootstrap

Use to create repository skeleton, tooling, CI baseline, docs, initial Goal and architecture references.

# architecture-change

Any change to core/UI boundary, Command model, renderer abstraction, plugin trust tier, PTND schema strategy or concurrency ownership.

# feature-standard

Default product feature implementation where no more specialized recipe dominates.

# engine-renderer

Renderer/compositor/shader/vector/raster backend changes with benchmark/visual evidence.

# ui-feature

New panel/dialog/control/tool presentation surface after UX/component contract exists.

# ui-review

Independent usability/visual/accessibility review of completed surface.

# design-system-foundation

Tokens, controls, themes, density, icon system and UI primitives.

# bug-fix

Reproduction-first defects; requires regression evidence.

# documentation-refactor

Authority restructuring, migration from old Rust/Slint docs, no content-loss mapping.

# security-review

Plugin/MCP/filesystem/parser/permission-sensitive changes.

# security-audit-release

Pre-release security artifact and supply-chain checks.

# release

Candidate build, changelog, artifacts, verification and publication preparation.

# github-issue

Translate known gap/bug/spec task into executable issue with acceptance/evidence.

# Orchestration rule

A Goal may combine recipes, but one is primary. Example new custom Photo tool = feature-standard + ui-feature; custom GPU effect = feature-standard + engine-renderer; new plugin permission = architecture-change + security-review.