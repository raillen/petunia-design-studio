# Glossary (EN ↔ PT-BR)

Canonical bilingual vocabulary. **English is normative**: when translations disagree, the English definition wins. Statuses: `preferred`, `deprecated` (kept, do not use in new work), `forbidden` (never emit).

## Core model

| EN (canonical) | PT-BR | Status | Definition |
| -------------- | ----- | ------ | ---------- |
| Surface | Superfície | preferred | Unified artboard/page/export region depending on context. |
| Artboard | Prancheta | preferred | Design surface for vector/layout composition; a Surface in artboard context. |
| Layer | Camada | preferred | ContainerRole in the single document tree; Layers panel role, not a parallel hierarchy. |
| Mask | Máscara | preferred | Non-destructive clipping/alpha control; source pixels preserved. |
| Symbol | Símbolo | preferred | Reusable layout/content definition; V1 replacement for Master Pages. |
| SurfaceTemplate | Modelo de superfície | preferred | Lightweight reusable surface layout built on Symbols. |
| Adjustment | Ajuste | preferred | Non-destructive, maskable, reorderable raster/color correction node. |
| Swatch | Amostra | preferred | Named reusable color definition in the semantic color system. |
| Persona | Persona | preferred | Design or Photo tool/panel/action composition over one shared document. |
| Profile | Perfil | preferred | Named mixed-workspace composition referencing semantic IDs. |
| Binding | Vínculo | preferred | Link between a document property and a variable-data field. |
| Record | Registro | preferred | Single row of a variable-data DataSource used in Data Merge generation. |

## Identity & automation

| EN (canonical) | PT-BR | Status | Definition |
| -------------- | ----- | ------ | ---------- |
| ObjectId | ObjectId | preferred | Typed stable identity for a document object; never a Vec index or pointer. |
| SurfaceId | SurfaceId | preferred | Typed stable identity for a surface. |
| ActionId | ActionId | preferred | Stable semantic identifier for a user operation (`aubrieta.*` read, `ptnd.*` emitted). |
| TextId | TextId | preferred | Semantic identifier for localizable UI copy; no literal strings in features. |
| IconId | IconId | preferred | Toolkit-independent namespaced icon identifier. |
| Action | Ação | preferred | User-level intent dispatched through the session. |
| Command | Comando | preferred | Validated mutation unit applied via DocumentMutator. |
| ChangeSet | ChangeSet | preferred | Atomic result of a Command: the only thing that touches storage. |
| DocumentMutator | DocumentMutator | preferred | Sole writer of document storage; all mutations pass through it. |
| EffectChain | Cadeia de efeitos | preferred | Typed ordered list of live modifiers evaluated over base geometry. |

## Product & format

| EN (canonical) | PT-BR | Status | Definition |
| -------------- | ----- | ------ | ---------- |
| Petunia Design Studio | Petunia Design Studio | preferred | Canonical application name (Design + Photo desktop app). Product names are not translated. |
| .ptnd | .ptnd | preferred | Canonical native file extension; open ZIP package format. |
| .aubrieta / .aubri | .aubrieta / .aubri | deprecated | Legacy native aliases; still read, never emitted on new saves. |
| Master Pages | Páginas-mestre | deprecated | Out of scope for V1; use Symbols + SurfaceTemplate. |
| .abrt | .abrt | forbidden | Rejected: collides with the Fedora/RHEL ABRT ecosystem. |
| .pds | .pds | forbidden | Rejected native extension candidate. |

## Process

| EN (canonical) | PT-BR | Status | Definition |
| -------------- | ----- | ------ | ---------- |
| Gauntlet | Gauntlet | preferred | The enforced verification suite (`cargo xtask gauntlet`); "gauntlet" is not translated. |
| Capability | Capacidade | preferred | Registry entry declaring what a feature provides; missing = disabled with reason. |
| Preflight | Pré-voo | preferred | Export-time checks (fonts, gamut, raster budget) before materializing bytes. |
| Bake | Consolidar (bake) | preferred | Explicitly freezing live modifiers into base geometry; always a user operation. The verb *bake* is kept untranslated in UI labels. |

> Source of truth for product terms: `docs/glossary.json` in the repo. This page renders that registry for humans; on conflict, the JSON wins for `preferred/forbidden` status.
