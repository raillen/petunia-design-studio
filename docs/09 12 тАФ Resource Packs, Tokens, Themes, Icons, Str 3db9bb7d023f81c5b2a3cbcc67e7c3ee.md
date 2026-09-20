# 09.12 — Resource Packs, Tokens, Themes, Icons, Strings & Configuration Formats

# Architecture decision

Aubrieta uses **JSON for runtime/interchange resource payloads** and **TOML for human-authored manifests/configuration**.

# Why JSON

Design tokens follow the stable DTCG 2025.10 JSON model where practical, enabling aliases, typed values and tooling interoperability. JSON also fits generated locale catalogs and semantic icon maps and has mature `serde_json` support.

# Why TOML

Pack/module/application manifests are configuration, not design-token interchange. TOML supports comments, readable tables and Serde, and is familiar in Rust/Cargo workflows.

# Resource bundle

Conceptual bundle:

```
aubrieta-pack/
  manifest.toml
  tokens/
    base.tokens.json
    dark.tokens.json
    light.tokens.json
  icons/
    icon-map.json
    *.svg
  strings/
    en-US.json
    pt-BR.json
  cursors/
  optional-assets/
```

# Manifest

Pack ID, semantic version, Aubrieta compatibility range, author/license/source, provided resource kinds, dependencies, fallback pack IDs and integrity metadata.

# Token model

Primitive → semantic → component tokens. Components may not reference palette literals directly. Aliases resolve with cycle detection. User packs may override semantic/component tokens but cannot introduce code execution.

# Text tokenization

Production UI uses `TextId` keys, never English literals. Locale JSON maps stable IDs to **Unicode MessageFormat 2** messages plus validated parameter schemas. MessageFormat 2 is the canonical V1 message model for plural/select/grammatical variation; Aubrieta must not invent a second ad-hoc interpolation/plural syntax. Parameters are named and typed/validated, while number/date/list formatting delegates to locale services/ICU4X-compatible formatters rather than string concatenation. `en-US` is the source/fallback catalog and `pt-BR` is the mandatory first complete translation for release.

# Icon tokenization

Actions/tools use `IconId`. Icon map resolves semantic ID to family asset; custom Aubrieta SVG has highest semantic-priority fallback, then selected family, then canonical fallback. Missing icon renders diagnostic placeholder only in development and safe fallback in release.

# Resolution order

Built-in canonical defaults → selected base pack → selected theme/icon/locale pack → user override layer. Invalid override falls back per key, not by crashing whole pack.

# Hot reload

Development mode can watch resource files and atomically swap a validated resource snapshot. Release user customization reload is optional and must not mutate running component state unsafely.

# Security

SVG resource parser forbids active/external content; resource paths are sandboxed to pack root; decompression/file counts are bounded.

# Tooling

Provide validator/compiler CLI that emits schemas, checks missing TextId/IconId/token references, contrast requirements, locale completeness, alias cycles, license metadata and produces an optimized runtime pack.

# Concrete resource-pack examples

## `manifest.toml`

```toml
[pack]
id = "org.example.aubrieta.midnight"
name = "Midnight Studio"
version = "1.2.0"
aubrieta = ">=0.1, <0.3"
author = "Example Studio"
license = "MIT"

[provides]
themes = ["midnight-dark"]
icons = ["midnight-icons"]
locales = ["pt-BR"]

[resources]
tokens = ["tokens/base.tokens.json", "tokens/dark.tokens.json"]
icons = "icons/icon-map.json"
strings = ["strings/pt-BR.json"]
```

## Design-token JSON

```json
{
  "surface": {
    "panel": {
      "$type": "color",
      "$value": "#20242B"
    }
  },
  "control": {
    "height": {
      "comfortable": {
        "$type": "dimension",
        "$value": { "value": 34, "unit": "px" }
      }
    }
  },
  "panel": {
    "background": {
      "$type": "color",
      "$value": "{surface.panel}"
    }
  }
}
```

## Semantic icon map

```json
{
  "schemaVersion": 1,
  "family": "midnight-icons",
  "icons": {
    "aubrieta.tool.pen": "pen-tool.svg",
    "aubrieta.action.boolean.union": "boolean-union.svg",
    "aubrieta.panel.layers": "layers.svg"
  }
}
```

## Locale catalog

```json
{
  "schemaVersion": 1,
  "locale": "pt-BR",
  "messages": {
    "aubrieta.action.export.title": "Exportar",
    "aubrieta.panel.layers.title": "Camadas",
    "aubrieta.error.resource_missing": {
      "message": "O recurso {name} não foi encontrado.",
      "parameters": { "name": "string" }
    }
  }
}
```

# Runtime representation

`TextId`, `IconId`, token paths and other resource IDs are namespaced extensible newtypes, **not closed enums**. Built-in constants are generated from canonical catalogs for typo safety, while plugins/packs can register their own namespace dynamically.

# UI/document separation

These design tokens describe **application UI presentation only**. They must not overwrite creative document swatches, object colors, ICC profiles or other document semantics. A purple UI theme never changes a purple object in the artwork.

# Standards and implementation references

- [DTCG Design Tokens Format Module 2025.10](https://www.designtokens.org/TR/2025.10/format/) — stable interchange format; design-token files are JSON and support typed tokens/aliases.
- [Design Tokens Community Group](https://www.designtokens.org/) — current stable/draft status and ecosystem interoperability.
- [Rust `toml` crate](https://docs.rs/toml/latest/toml/) — Serde-compatible TOML parser/serializer; TOML remains the human-authored configuration/manifest format.

# Resource service boundary

All feature/UI code resolves presentation resources through `aubrieta_resources`/`aubrieta_i18n` interfaces. GPUI is a consumer adapter; it does not own the canonical catalogs.

Conceptual API surface:

```
ResourceSnapshot
- generation
- resolve_text(TextId, Locale, Arguments) -> ResolvedText
- resolve_icon(IconId, IconContext) -> IconAssetRef
- resolve_token(TokenId, ThemeContext) -> TypedTokenValue
- resolve_cursor(CursorId) -> CursorAssetRef
- resolve_help(HelpTopicId, Locale) -> HelpTarget
```

Resolution returns immutable generation-bound data. Feature modules never retain mutable pack internals.

# Atomic snapshot model

Pack/theme/icon/locale changes build and validate a **new complete ResourceSnapshot** off the UI critical path, then atomically publish one generation. Components either render generation N or N+1; they must not observe half a theme with old strings/new icons.

A failed pack reload leaves the previous valid snapshot active and emits structured diagnostics.

# Namespace policy

Built-in semantic IDs use `aubrieta.*`. Third-party resources/plugins use a reverse-domain or otherwise registered globally unique namespace such as `org.example.plugin.*`.

Validation enforces:

- syntactically valid normalized IDs;
- namespace ownership from manifest;
- no registration outside owned namespaces except explicitly permitted override slots;
- deterministic collision failure;
- reserved `aubrieta.*` namespace cannot be claimed by external packs.

# Pack dependency graph

A resource pack may depend on other packs by PackId + compatible semantic version range. Required dependency graph must be acyclic. Resolution computes a deterministic topological order before loading payloads.

Missing/incompatible dependency disables only the affected pack and falls back to previous/built-in resources; it cannot make Aubrieta fail to boot.

# Override classes

Distinguish clearly:

1. **base provider** — contributes new namespaced IDs;
2. **theme/icon/locale selection** — overrides approved built-in presentation slots;
3. **user override** — highest-precedence safe presentation values;
4. **plugin-owned resources** — only inside plugin namespace unless host explicitly maps them into a contribution.

A theme pack cannot override Action/Command IDs, capability permissions, document schemas or accessibility semantic meaning.

# Token typing

`TokenId` resolves to a typed union, not unvalidated JSON at component call sites. Required token kinds include color, dimension, duration, easing, number, opacity, typography reference, border/shadow structure and other Design System-approved types.

Alias resolution validates type compatibility. A color token cannot alias a dimension token. Unknown token types from a newer optional pack are preserved/ignored according to compatibility policy but cannot be coerced unpredictably.

# Token units

Dimensions use explicit units supported by the UI token schema (`px`/logical UI units as defined by Design System). Token values are presentation metrics before platform scale; GPUI adapter performs scale-aware physical-pixel handling for hairlines/HiDPI. Document units such as mm/in/pt are not design-token units unless a component specifically represents document data.

# Theme validation

Before activation validate at least:

- all required semantic roles present or safely fallbackable;
- alias graph acyclic;
- valid numeric/color ranges;
- focus/selection/error states remain distinguishable;
- contrast-critical text/control combinations meet Aubrieta accessibility thresholds;
- disabled/selected states do not rely on color alone;
- resource count/depth/size limits.

Theme validation reports per-token failures and keeps built-in value for invalid/missing overrides.

# Icon asset contract

SVG assets are parsed/sanitized into a safe internal vector asset representation before publication. Disallow scripts, event handlers, external references, remote fonts/images, unsafe filters/URLs and unbounded complexity.

Icon metadata may define optical box/baseline/variant information, but accessible label comes from Action/Tool/Text semantics — never SVG title/filename.

A semantic icon provider returns one of:

- Aubrieta custom exact icon;
- selected family mapping;
- built-in Lucide fallback;
- explicit safe generic placeholder if no meaningful icon exists.

A missing icon cannot replace a text-required action with an empty button.

# Locale catalog schema

Each locale catalog declares:

- `schemaVersion`;
- BCP-47 locale tag;
- source/fallback locale relationship where applicable;
- message map `TextId -> MessageFormat2 source/AST-ready representation`;
- typed parameter schema;
- translator notes/context metadata in source tooling as needed.

Runtime compiler/validator parses MessageFormat 2 before publishing the catalog. Invalid message syntax/parameter mismatch falls back per TextId to `en-US` and records diagnostic.

# Locale fallback

Canonical order:

1. exact selected locale, e.g. `pt-BR`;
2. configured language fallback when explicitly supported, e.g. `pt`;
3. `en-US` canonical fallback;
4. development diagnostic placeholder only if canonical fallback is itself missing.

Release CI treats a missing built-in canonical `en-US` TextId as error.

# Translation completeness policy

For product release, `pt-BR` must be complete for all **shipping user-visible built-in TextIds** unless a page/feature is explicitly excluded from that release. Plugin locales are owned by plugins and may fall back independently.

Generated/diagnostic developer-only strings still use TextId where user-visible, but translation gates may classify them separately from release-critical product UI.

# MessageFormat 2 argument safety

Call sites pass named typed arguments; positional string concatenation is prohibited. Catalog schema/tooling validates that:

- required arguments exist;
- argument types match formatter/select usage;
- no unused/misspelled argument names hide errors;
- user-provided text is data, not interpreted as MessageFormat source.

# Formatting services

Locale-aware numbers/dates/lists/units route through `aubrieta_i18n`/ICU4X-compatible typed formatting. Serialization, file-format grammar, stable IDs and CLI machine output remain locale-invariant unless specifically human-formatted.

# Configuration format separation

TOML configuration/manifests are loaded into typed structs with explicit defaults/migrations. Runtime code never performs scattered string-key lookup into arbitrary TOML tables.

JSON resource payloads are similarly validated/compiled into typed runtime structures. Parsed `serde_json::Value` is a boundary/staging type, not the long-lived API for components.

# User preference files

User configuration is human-editable TOML only where hand editing is intentionally supported. Corrupt config follows 09.23 layered-settings recovery: quarantine/diagnose invalid entries, keep safe defaults and preserve unknown newer keys where feasible.

# Pack installation

Import/install flow:

1. acquire local package/source;
2. parse manifest under limits;
3. verify namespace/version/dependencies;
4. inspect license/provenance;
5. sanitize/validate all payloads;
6. compile catalogs/token aliases;
7. run accessibility/theme checks;
8. stage in user resource directory;
9. atomically activate/update resource registry.

Installation never executes scripts/hooks from the pack.

# Trust/signatures

Ordinary resource packs are **data-only and untrusted by default**. Cryptographic signatures may later establish provenance/update trust, but signature presence does not grant code/plugin capabilities. A resource pack cannot become executable merely because it is signed.

# Licensing/provenance

Manifest requires license/source/author metadata appropriate to distributed assets. Build/release tooling collects active bundled third-party icon/font/resource obligations into notices. An invalid/missing license in a user-imported pack can warn/block redistribution workflows according to policy but does not fabricate a license.

# Resource limits

Set checked limits for:

- pack file count/total bytes;
- JSON/TOML depth/string/token count;
- SVG node/path complexity;
- message length/AST complexity;
- alias/dependency depth;
- icon raster/vector dimensions where applicable.

Hostile packs fail validation before expensive UI instantiation.

# Cache/compiled pack

Release builds may compile JSON/TOML/SVG/message resources into optimized binary/runtime representations. Compiled output is derived from canonical source catalogs and carries schema/compiler version. It does not become a second manually authored source of truth.

# CI no-hardcode enforcement

Repository checks should maintain allowlisted directories for canonical literals and flag likely violations elsewhere:

- user-visible string literals in UI/feature code;
- direct `.svg`/icon file paths;
- literal theme colors;
- raw spacing/radius/motion constants;
- unknown TextId/IconId/TokenId references;
- orphan catalog keys never referenced/generated.

False-positive suppression requires an explicit annotation/comment with rationale; suppression itself is reviewable.

# Code generation

Generate built-in Rust constants/types from canonical registries:

```
texts::ACTION_EXPORT_TITLE
icons::BOOLEAN_UNION
tokens::SURFACE_PANEL
settings::UI_DENSITY
```

Generation validates uniqueness and updates documentation reference tables. The generated constants are convenience wrappers around extensible runtime IDs, not closed enums.

# Resource documentation

VitePress/generated reference publishes:

- resource-pack manifest schema;
- token domains/types;
- semantic built-in TextId/IconId/TokenId registries;
- theming examples;
- localization/MessageFormat 2 authoring guide;
- validation errors and fallback rules.

Examples are machine-tested by the same validator used by the application.

# Additional gauntlets

- dependency/alias cycle rejection;
- malformed pack leaves previous snapshot active;
- switch theme+icon+locale repeatedly during active dialogs/panels;
- pseudo-localized 50% expansion;
- RTL/CJK catalog plus IME UI;
- hostile SVG/external reference rejection;
- missing/corrupt individual resource falling back per key;
- plugin namespace collision;
- resource pack unable to alter Action meaning/permissions/document colors;
- generated constants/catalog parity and VitePress example validation.