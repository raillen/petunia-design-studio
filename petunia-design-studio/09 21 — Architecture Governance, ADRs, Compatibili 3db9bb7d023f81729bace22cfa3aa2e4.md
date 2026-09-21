# 09.21 — Architecture Governance, ADRs, Compatibility, Implementation Sequence & Definition of Done

# ADR policy

High-impact choices use Architecture Decision Records with Context, Constraints, Options, Decision, Consequences, Revisit Trigger and Supersedes. Libraries are not architectural facts merely because prototypes use them.

# Contract ownership

Every public contract names an owner module and compatibility policy. Cross-crate domain interfaces stay small. A module cannot expose internal storage merely for convenience.

# Compatibility dimensions

Native document schema, plugin component API, MCP protocol, resource-pack schema, config schema and workspace-layout schema are versioned independently.

# Implementation sequence

For each subsystem: specification → minimal domain types/contracts → reference tests/fixtures → vertical slice → observability → performance baseline → failure/security tests → extension points → optimization. Avoid optimizing before semantic oracle exists.

# Definition of Done

Feature requires: functional spec, architecture contract, semantic IDs/TextId/IconId/tokens, Action/Command mapping, undo/cancel policy, serialization impact, capability dependencies, diagnostics, security/failure modes, tests, performance budget, accessibility/UI mapping if visible, automation surface if appropriate and documentation links.

Additionally, for the current Code-Agent/Living-Documentation model a feature is not complete until applicable gates pass:

- headless/domain behavior works without GPUI initialization;
- optional module declares detach/unload behavior and survives missing-provider tests;
- visible UI follows 08.20 usability and 08.21 token/portability contracts;
- no user-facing text/icon/style/help/a11y resource is hard-coded outside canonical catalogs/tokens;
- Plugin/MCP surfaces reuse shared semantic contracts and include safety/version documentation;
- English repository documentation is updated first and pt-BR is synchronized;
- VitePress builds with links/examples/generated reference valid;
- any new public/persisted/security/UI-paradigm decision has an ADR or explicit accepted specification.

# No hidden coupling gate

Review asks: can the feature module be disabled or replaced without recompiling unrelated domain logic? If not, dependency must be justified as foundational and documented.

For optional modules, review additionally asks whether disable/unload leaves dangling jobs/listeners/panels/actions, corrupts workspace persistence, loses opaque document data or forces unrelated modules to know the removed implementation. Any of those outcomes is a modularity defect.

For UI work, review asks whether replacing GPUI would require changing domain/application semantics. If yes, the toolkit boundary has leaked and must be repaired through the semantic ports/bridge defined in 09.27.

# API evolution

Prefer additive changes; deprecate before removal. Internal API can evolve faster, but persisted/protocol/plugin contracts require migration/version policy.

# Documentation drift

Tests/CI should generate registry catalogs and compare against documentation/resource IDs. Missing registered action/panel/tool/text/icon in the documented atlas is a release-blocking documentation drift issue for stable surfaces.

The living documentation pipeline in section 12 adds VitePress build/link checks, code-example tests, generated Plugin/MCP/schema references, English↔pt-BR freshness checks and code-agent retrieval tests. Documentation is part of the compatibility surface, not an after-the-fact summary.

# Re-audit

Repeat 09.0 at milestone boundaries and after adding any new engine, format, plugin class or platform service.

# Canonical authority order

When documentation appears to conflict, code agents and reviewers resolve authority in this order unless a page explicitly states a narrower superseding contract:

1. accepted ADR that explicitly supersedes an older decision;
2. Product Charter/non-negotiable project invariants for product scope;
3. Architecture Atlas for internal contracts/ownership/security/persistence;
4. Functional Engine Atlas for user-visible/domain behavior and V1 status;
5. Interface Atlas for presentation/interaction/usability;
6. Code Agent Handbook for implementation process/documentation workflow;
7. reference/prior-art/research pages;
8. historical VectorVonDoom/Petunia discussions only as noncanonical context.

A newer edit date alone does **not** make a lower-authority page supersede a canonical decision. Supersession must be explicit.

# Decision/status vocabulary

Use formal statuses rather than prose ambiguity:

- `ACCEPTED_V1` — architecture/product decision fixed for V1;
- `V1_REQUIRED` — feature/contract required for V1 release;
- `V1_IF_FOUNDATION_READY` — desired V1 feature allowed only after named prerequisite/gate;
- `POST_V1` — intentionally outside V1;
- `EXPERIMENTAL` — prototype/research, not product promise;
- `REFERENCE_ONLY` — prior art/background;
- `DEPRECATED` — supported only for migration window;
- `SUPERSEDED` — no longer authoritative, with replacement link;
- `OPEN_ADR` — unresolved high-impact decision with safe behavior/evidence/revisit trigger.

Words such as `later`, `future`, `planned`, `candidate`, `maybe`, `if supported` and `when implemented` are explanatory only and cannot establish roadmap status without one of the formal states.

# ADR registry consistency gate

09.26 is the index of genuinely open/deferred decisions. An accepted ADR must be removed from its open section and added to resolved history in the same documentation change. CI/review should fail when:

- an ADR page says `ACCEPTED` while registry calls it open;
- two accepted ADRs claim the same decision without supersession link;
- a canonical page still describes an accepted decision as pending;
- a code dependency is selected while its architecture status remains `OPEN_ADR` without an explicitly safe abstraction.

# Contract change protocol

A semantic change must update all affected authorities in one change set:

1. ADR if architectural/high-impact decision changes;
2. Product/Functional scope status;
3. Architecture contract/data/schema/version impact;
4. UI/UX if user-visible;
5. Plugin/MCP/public API/schema impact;
6. en-US docs + pt-BR translation/VitePress references;
7. tests/fixtures/migrations/gauntlets.

A code change may not silently become the new specification because tests happen to pass.

# Compatibility classification

Every public/persisted change is classified before implementation:

- `InternalOnly`;
- `AdditiveCompatible`;
- `BehaviorCompatibleButObservable`;
- `Deprecation`;
- `BreakingProtocol/API`;
- `SchemaMigrationRequired`;
- `SecurityPermissionChange`;
- `UIInteractionContractChange`.

The classification determines required version bump/migration/ADR/docs/release notes.

# Definition of Ready for implementation

Before a code agent starts a nontrivial feature, the microcontext should establish:

- canonical user goal/status;
- owning module/crate and dependencies;
- domain/data/property/action IDs;
- mutation/undo/cancel semantics;
- persistence/version impact;
- concurrency/job ownership;
- error/security constraints;
- UI/UX/accessibility if visible;
- Plugin/MCP exposure;
- acceptance tests/performance budget;
- documentation pages to update.

If one of these is materially ambiguous, the agent resolves/document it before broad implementation rather than encoding a guess across many files.

# Definition of Done evidence packet

A completed feature/change should produce a machine/human-readable implementation evidence summary containing:

- changed semantic IDs/contracts;
- tests/gauntlets run and results;
- benchmark/performance impact;
- security/permission impact;
- migration/compatibility result;
- UI/accessibility/localization result;
- module detach/headless result if applicable;
- Plugin/MCP conformance result;
- docs/VitePress en-US + pt-BR update status;
- remaining explicitly deferred issues with formal status.

This packet can live in PR description/release artifact and is the basis for code-agent handoff.

# Architectural debt rule

Temporary architectural exceptions require an issue/ADR/debt record with:

- exact violated rule;
- reason/constraint;
- bounded scope;
- owner/removal trigger;
- test preventing spread where possible.

`TODO refactor later` is not enough to legitimize a forbidden dependency edge or hard-coded UI resource.

# No speculative abstraction rule

The no-gap documentation requirement does not mean implementing abstraction for hypothetical features. Prefer the smallest architecture satisfying accepted V1 + documented extension seams. New generic layer must have at least two concrete use cases or a foundational boundary/security reason. Agents must not create elaborate frameworks solely because future plugins *might* need them.

# Architecture review checklist for PR/code-agent changes

Reviewers/agents answer:

- Did dependency direction remain valid?
- Did any toolkit/third-party type leak across boundary?
- Is mutation still centralized?
- Is persisted/public schema version handled?
- Is failure atomic/recoverable?
- Is long work cancelable/stale-safe?
- Can optional module detach?
- Are strings/icons/tokens semantic?
- Does headless path still work?
- Are Plugin/MCP surfaces equivalent and permissioned?
- Did en-US/pt-BR/VitePress update with behavior?
- Is any “future/later/TBD” ambiguity newly introduced?

# Milestone freeze

Before declaring an architecture milestone frozen, run:

- 09.0 completeness audit;
- 13 page-by-page conformance ledger review;
- forbidden dependency graph check;
- open ADR registry consistency;
- schema/migration fixture suite;
- headless/mock-shell conformance;
- module detach matrix;
- docs retrieval test from a clean code-agent context.

Freeze means “canonical enough to implement against,” not “never changes”; changes after freeze require the protocol above.