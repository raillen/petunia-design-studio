# 09.16 — Localization, Locale Formatting, Translation Workflow & Text Resource Architecture

# Principle

No production user-visible sentence, menu label, tooltip, error, command title or accessibility label is hard-coded in feature code. Code references namespaced `TextId`.

# ID design

Runtime IDs are extensible namespaced strings/newtypes, e.g. `aubrieta.action.export.title`, `plugin.vendor.tool.foo.tooltip`. Built-in IDs are generated as typed constants for compile-time convenience; the registry is not a closed Rust enum.

# Locale catalogs

JSON per locale with schema version and stable message IDs. English (`en-US`) is the canonical source/fallback locale, not an implementation literal. **Brazilian Portuguese (`pt-BR`) is the mandatory first complete additional locale for release**: every shipping built-in user-visible TextId must have a validated pt-BR translation or the feature is explicitly excluded from that release scope.

# Message parameters

Messages declare named typed parameters. Formatting is performed by locale service using ICU4X-compatible locale data for numbers, dates, lists, plural categories and directionality. Never concatenate translated fragments to build sentences.

# Plurals/selects

**Resolved syntax:** Aubrieta standardizes on **Unicode MessageFormat 2 (MF2)** semantics/syntax for dynamic localizable messages. MF2 is now a stable Unicode/CLDR standard and gives us named variables, plural/select matching, locale-aware formatting and a cross-tooling data model instead of inventing a Aubrieta-only expression language.

Runtime code remains behind a Aubrieta localization facade. Rust MF2 libraries are still comparatively young, so the dependency may change after conformance testing without changing catalog syntax. ICU4X remains the locale-data/number/date/plural service where it fits. Catalog compilation validates MF2 syntax, typed parameters, selectors and required fallbacks before shipping.

# Fallback

Exact locale → language fallback → built-in source locale → diagnostic placeholder only in dev. Missing one key does not invalidate the entire pack.

# Layout implications

Pseudo-locales for expansion (+30–60%), diacritics and RTL mirror testing. Do not bake shortcut strings into translated labels; menu renderer composes them.

# Translation workflow

Extract/generate source catalog from registered TextIds; diff added/removed keys; translator comments/context; locale completeness report; lint placeholder mismatches; screenshot/semantic smoke tests under each release locale.

# Plugins

Plugins supply namespaced string catalogs and fallback locale. Host may override common standardized terms only through explicit shared IDs, never accidental key collision.

# Tests

Pseudo-localization, RTL, plural categories, missing parameters, long filenames/error arguments, mixed Latin/CJK/Arabic UI labels and locale hot switch where supported.

# Product localization vs documentation translation

This page governs **runtime product strings and locale formatting**. It is separate from the documentation-language policy in section 12.

- Runtime UI catalogs use `TextId` + MF2/locale services and may support many locales independently.
- Repository/VitePress documentation uses **English (`en-US`) as canonical source** with **Brazilian Portuguese (`pt-BR`) as mandatory release translation**.
- API identifiers, schemas and code symbols remain invariant in both systems.

Code agents must not store documentation paragraphs inside runtime locale catalogs or use VitePress translation files as application UI resources.