# 12.1 — Source of Truth, Canonical Documentation, Language Policy & Drift Resolution

# Canonical roles

Aubrieta deliberately separates **decision authority** from **published implementation reference** to avoid one giant mutable document becoming every kind of truth.

- **Notion notebook:** product intent, architecture decisions, UX contracts, capability boundaries, ADRs, implementation acceptance criteria.
- **Repository docs:** branch/release-specific implementation guide and API reference consumed by contributors/code agents.
- **VitePress site:** rendered publication of repository docs.
- **Schemas/tests/generated catalogs:** executable evidence for machine-readable contracts.

# Conflict resolution

If two canonical pages disagree, prefer the more specific and more recently accepted explicit decision, then update the stale page. Do not leave contradictory guidance with a note saying both are valid unless they truly describe alternatives.

If code/tests disagree with docs, determine whether code is a bug or the spec is stale. Public/persisted contract changes require ADR/compatibility review before documentation is rewritten to legitimize implementation drift.

# Normative wording

Use stable requirement words consistently:

- **MUST / MUST NOT** — release-blocking invariant;
- **SHOULD / SHOULD NOT** — strong default, deviation requires documented reason;
- **MAY** — optional implementation choice;
- **Candidate / Research** — not accepted architecture.

Avoid `probably`, `maybe`, `TBD later` in implementation-grade sections without a tracked ADR/revisit trigger.

# English canonical + Portuguese mirror

Repository structure keeps parallel locales. English is authored first because it is the canonical reference for APIs, external contributors and LLM/code-agent tooling. pt-BR follows immediately in the same change unless explicitly marked temporarily stale during a draft branch.

Translation rules:

- preserve code blocks and identifiers exactly unless comments are intentionally localized;
- preserve heading anchors where the site requires stable cross-locale links;
- preserve normative strength (MUST remains MUST);
- do not translate file extensions, ActionId, PropertyId, schemas or CLI flags;
- translator notes never change product behavior.

# Freshness metadata

Each translated document should be traceable to its English source through a content hash, source commit or frontmatter field such as `source_revision`. CI can flag translation drift when the source changes.

# One concept, one canonical term

Maintain glossary entries for terms such as Document, Surface, Layer, Object, PixelLayer, Image, Mask, Clip, Symbol, Adjustment, Effect, Persona, Workspace, Action, Command, Capability, Resource Pack, Plugin, MCP, Binding and Data Source.

Aliases may aid search but should not create competing product vocabulary.

# Historical material

Historical VectorVonDoom/Petunia decisions remain reference/prior art only. Historical pages must be visibly labeled so agents do not treat obsolete names/architecture as current contracts.

# Documentation ownership

Every major subsystem names:

- owning module/team role;
- canonical Notion page;
- repository docs path;
- generated schema/reference source where applicable;
- compatibility version;
- last architecture review milestone.

# Drift detection

CI/review should detect:

- registered ActionId/PropertyId/CapabilityId absent from generated reference;
- documentation links to removed IDs;
- examples that no longer compile/run;
- untranslated new English pages/sections;
- VitePress broken links;
- stale generated API pages;
- undocumented persisted schema/version change;
- undocumented new user-visible component or permission.

# Deprecation docs

When removing/renaming APIs or user-facing concepts, document replacement, migration path, first deprecated version, planned removal version/revisit condition and compatibility effect. Never silently remove a documented API surface.