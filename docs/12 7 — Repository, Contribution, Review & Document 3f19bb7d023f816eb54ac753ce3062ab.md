# 12.7 — Repository, Contribution, Review & Documentation Maintenance Rules for Agents

# Branch/PR discipline

One Goal has bounded changes. Avoid drive-by refactors unrelated to acceptance criteria. Generated/vendor files changed only by owning workflow.

# Commit/PR

Commit messages name semantic change. PR body maps criteria, affected architecture/docs, tests/evidence, migrations and known limitations. Large native/UI changes split by dependency order, not arbitrary file count.

# Review

Reviewer receives locked criteria + diff + evidence. Comments distinguish blocker, correctness, architecture, security, performance, accessibility and suggestion.

# Documentation

Any Action/Tool/Panel/Property/schema/API/file-format/permission change lists canonical doc pages updated. Documentation Maintainer owns synchronization after verification.

# Compatibility

Do not rename stable IDs or schema fields for aesthetics. Deprecate/migrate explicitly.

# Hygiene

No commented-out dead implementations, debug prints, secret test credentials, local absolute paths or unchecked generated artifacts.

# Ownership

CODEOWNERS/module owners may be introduced as team grows; architectural boundaries are more important than folder ownership alone.