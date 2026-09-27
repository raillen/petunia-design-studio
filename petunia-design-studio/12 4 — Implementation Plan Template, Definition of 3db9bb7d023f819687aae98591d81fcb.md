# 12.4 — Implementation Plan Template, Definition of Ready, Definition of Done & ADR Triggers

# Definition of Ready

A feature is ready for implementation when the agent can answer:

- what user/system problem is solved;
- canonical source pages;
- in/out of scope;
- responsible module/capability;
- public/persisted schema impact;
- action/command/property model;
- failure/recovery behavior;
- security/permission implications;
- UI/UX/accessibility implications;
- tests and measurable acceptance criteria.

If several answers are unknown and materially affect architecture, specification work comes first.

# Implementation plan template

```
Goal
Non-goals
Canonical references
Affected modules/crates
Capabilities required/provided
Data model / schemas
Actions / Commands / Transactions
Read/query APIs
Jobs / concurrency / cancellation
Persistence / migrations
Diagnostics / failure recovery
Security / permissions
UI presentation models / token IDs
Accessibility / localization
Plugin/MCP exposure
Tests / fuzz / property tests
Performance budgets
Documentation EN + pt-BR
Rollout / compatibility / migration
```

# ADR triggers

Create/update an ADR for changes involving:

- core dependency direction;
- persistent/native schema;
- public plugin/MCP API;
- plugin runtime language;
- new permission/security boundary;
- replacement of canonical library/backend;
- new UI interaction paradigm/design-system primitive with broad impact;
- compatibility break;
- new external service/protocol;
- FFI/unsafe architecture;
- irreversible data migration.

Do not create ADRs for trivial local refactors.

# Feature Definition of Done

A complete feature has:

- documented semantic behavior;
- correct domain/application implementation;
- authoritative Action/Command path;
- undo/cancel semantics;
- capability registration and detach behavior;
- persisted schema/migration if needed;
- structured diagnostics;
- security/resource limits;
- unit/contract/property/fuzz tests as applicable;
- performance baseline;
- UI all states if visible;
- keyboard/accessibility/localization coverage;
- semantic resources (TextId/IconId/tokens/help);
- plugin/MCP discoverability where appropriate;
- English docs + pt-BR translation;
- VitePress build success;
- no unresolved architecture TODO hidden in stable code.

# Module Definition of Done

A new module additionally documents:

- `ModuleId`;
- provided/required/optional capabilities;
- lifecycle;
- contributions;
- owned persisted keys/data;
- unload/disable behavior;
- jobs/thread ownership;
- resource budgets;
- permission requirements;
- observability hooks;
- version/compatibility policy;
- test fixture disabling/removing module without collateral breakage.

# UI Definition of Done

Visible work additionally passes 08.20/08.21 and 08.16: all states, discoverability, tokenization, keyboard, accessibility, localization, density/theme matrix, performance and semantic automation hooks.

# API Definition of Done

Public plugin/MCP API additionally has stable schema, examples, permission model, error model, compatibility notes, tests and generated reference.

# Completion language

Agents should say `implemented for current milestone` rather than `finished forever` when deferred post-V1 functionality remains. Scope boundaries belong in the completion report.

# Implementation dossier requirement

Before a substantial change moves from planning to implementation, create/update a version-controlled **implementation dossier** (path determined by repository docs conventions/Prumo configuration) containing at minimum: Goal, scope status, canonical references, dependency/capability map, planned Commands/Properties/resources, risk register, test/gauntlet plan, compatibility/migration impact, documentation impact manifest and completion evidence.

The dossier is not a replacement for canonical Atlas pages; it is the task-specific bridge from accepted contracts to code. Close/archive it with final results and residual risks so later agents can reconstruct why the implementation looks the way it does.