# 12.9 — Change Impact Matrix: What Agents Must Update for Every Kind of Feature Change

<aside>
🧬

This matrix prevents local code changes from leaving registries, UX, automation, translations or living documentation inconsistent.

</aside>

# New Action

Update/check: ActionId registry; title/description TextIds; optional IconId; context/enabled/checked state; default shortcut/key context; menu/toolbar/palette eligibility; Command binding; undo/transaction; telemetry/test name; MCP generic discovery; plugin exposure policy; EN + pt-BR docs; tests.

# New Command/domain mutation

Update/check: typed input/output; preconditions; authorization/capability; transaction boundary; ChangeSet/invalidation classes; undo/redo; persistence/migration impact; diagnostics; headless tests; plugin/MCP invocation path; performance/security limits.

# New Property/Parameter

Update/check: PropertyId/ParameterId; schema descriptor; type/unit/range/default/mixed/null semantics; editor hint; invalidation; bindability; automation visibility; serialization/version; generic inspector; Data Merge coercion if bindable; MCP schema; docs/examples.

# New Tool

Update/check: ToolId; activation Action; IconId/CursorId/TextIds; input state machine; selection rules; normalized pointer/pen contract; preview/transaction/commit/cancel; snapping; overlays; context toolbar; Properties schema; accessibility/help; module capability; MCP/plugin automation equivalent when appropriate; 10.x semantics; 08.x UX; gauntlet fixture; EN/pt-BR docs.

# New Panel/Inspector

Update/check: PanelId; module contribution; capability prerequisites; presentation model; no raw domain mutation; docking/min/preferred sizes; empty/loading/error/disabled states; keyboard/focus/a11y; persistence scope; TextId/IconId/help; virtualization/performance; workspace missing-panel recovery; component gallery; docs.

# New reusable UI component

Update/check: semantic purpose; token dependencies; complete state matrix; input/focus semantics; accessibility; localization/density/HiDPI; gallery story; visual + semantic regression; toolkit-neutral contract if it represents product semantics; documentation in 08.5/08.21.

# New design token/theme resource

Update/check: TokenId/path; primitive/semantic/component level; type; default Dark/Light/System values; aliases/fallback; validation/contrast implications; DTCG-compatible JSON where applicable; gallery coverage; all required themes; docs/pt-BR if user-facing semantics change.

# New user-visible string

Update/check: TextId; English source message and translator context; MF2 parameter schema if dynamic; pt-BR runtime translation if shipped; pseudo-localization; accessibility usage; documentation terminology/glossary consistency. Do not use display text as identity.

# New icon/cursor/asset

Update/check: semantic resource ID; fallback mapping; optical sizing/states; accessible label comes from action semantics; licenses/provenance; theme tint; HiDPI; resource-pack validation; gallery/browser.

# New module/capability

Update/check: ModuleId/CapabilityId; required/optional/provided contracts; lifecycle; contribution list; permissions; jobs; settings/resources; document extension data; migration owner; disable/unload behavior; missing-provider behavior; collision policy; detach CI fixture; generated capability docs.

# New plugin permission

Update/check: stable permission ID; least-privilege scope; risk category; manifest syntax; install/update consent wording; settings/revocation UI; broker implementation; audit/diagnostics; denial tests; escalation-on-update behavior; Plugin SDK and security docs; MCP interaction if shared.

# New MCP method/helper

Update/check: stable method ID; schema; permission; revision/transaction/idempotency behavior; dry-run if high impact; structured errors; pagination/job behavior; generic semantic primitive mapping; minimal+advanced examples; EN/pt-BR docs; conformance with direct Action/Command outcome.

# New native-format field/object/schema

Update/check: schema version; backwards/forwards behavior; default/missing semantics; migration; unknown-data preservation; canonical JSON schema; fixtures; corruption/security limits; compatibility/interchange degradation; documentation and external reference implementation impact.

# New importer/exporter

Update/check: capability descriptor; sniffing/detection; fidelity/degradation mapping; preflight; color/text/font/resource policies; hostile-input limits/fuzz corpus; cancellation/atomic output; plugin contribution schema if applicable; fixtures with external tools; docs.

# New background job

Update/check: JobKind; priority; cancellation points; revision/fingerprint stale-result policy; progress phases; memory/resource estimate; module ownership/unload; diagnostics; UI Background Tasks surface; MCP job representation; saturation/backpressure tests.

# New setting/preference

Update/check: SettingId; scope (app/workspace/document/session); default; validation; persistence/version; live/restart behavior; reset; UI category/search; localization/help; plugin visibility; migration; configuration docs.

# New documentation page

Update/check: canonical category/owner; English source first; pt-BR mirror; navigation/sidebar; links; source_revision/freshness metadata where used; search terms/glossary; code examples/tests; accessibility; VitePress build.

# Dependency/library change

Update/check: adapter boundary; license/SBOM; maintenance/security status; platform build implications; feature flags; binary/runtime size; replacement/fallback strategy; benchmarks/tests; architecture page and lockfile/toolchain docs.

# Removal/rename

Update/check: deprecation status; replacement; aliases/migration; persisted/public compatibility; docs redirects/links; translation; generated catalogs; plugin/MCP compatibility; fixtures; release notes.

# Agent completion rule

Before finalizing a change, the code agent identifies its categories from this matrix and explicitly reports which rows were applicable and which artifacts were updated. Missing an applicable cross-cutting artifact is an incomplete implementation, not a later cleanup task.

# New Plugin SDK method/type/helper

Update/check: semantic API ownership; Lua binding; capability/permission; typed value mapping; transaction/job behavior; quotas; structured errors; beginner and advanced examples; headless plugin fixture; generated SDK reference; EN/pt-BR cookbook/reference; deprecation/version metadata; MCP equivalent only when semantically appropriate (do not create parity merely for naming symmetry).

# New persisted configuration/workspace field

Update/check: owning SettingId/schema; legal scope; default/missing behavior; config/workspace schema version; migration; invalid manual edit recovery; export/import preferences behavior; privacy/secret classification; UI control if exposed; docs and fixtures.

# New Persona/workspace contribution

Update/check: capability composition only; no new document semantics hidden in Persona; tools/panels/actions included; workspace persistence/fallback when capability absent; accessibility/shortcut conflicts; UI atlas; no-dirty Persona-switch tests.

# New brand/package/file association

Update/check: 11 naming contract; installer/application metadata; file extensions/MIME/UTI/desktop associations; CLI/package IDs; semantic namespace compatibility; docs/site; update/migration strategy. Never perform a blind global rename of persisted/public IDs.

# Documentation impact manifest

For every substantial task, produce a small machine/human-readable checklist enumerating applicable rows from this matrix and the actual artifacts updated. Prumo-generated task/dossier output should include this manifest or equivalent. `Not applicable` entries require a reason when the row would normally be expected for the change category.