# 12.3 — Living Documentation Pipeline: VitePress, i18n, CI/CD, Versioning & Site Information Architecture

<aside>
🌐

Aubrieta maintains a living documentation website built with **VitePress** from version-controlled Markdown. The site is a published projection of implementation documentation, not a manually maintained second source.

</aside>

# Suggested repository layout

```
docs/
  .vitepress/
    config.mts
    theme/
  en-US/
    index.md
    getting-started/
    user-guide/
    concepts/
    ui-ux/
    architecture/
    implementation/
    api/
      core/
      plugins/
      mcp/
    formats/
    security/
    testing/
    contributing/
    adr/
    troubleshooting/
    glossary.md
  pt-BR/
    ... mirrored structure ...
  public/
```

Exact folder names may be simplified, but locale parity and semantic categories should remain predictable.

# Site navigation

Primary navigation should answer different audiences quickly:

- **Use Aubrieta** — user workflows;
- **Understand Aubrieta** — concepts/document model;
- **Build Aubrieta** — architecture/implementation/contributing;
- **Extend Aubrieta** — Plugin SDK;
- **Automate Aubrieta** — MCP;
- **Reference** — schemas/formats/API/diagnostics;
- **Decisions** — ADRs/changelog/migrations.

Do not mix end-user tutorials with internal engine notes in the same navigation level.

# VitePress locale model

Configure `en-US` as canonical/default locale and `pt-BR` as first translated locale. Locale switch should preserve equivalent page when available and fall back clearly when translation is missing/stale.

# Frontmatter contract

Documents may carry metadata such as:

```yaml
title: Plugin permissions
status: stable
since: 0.1
source_revision: <commit-or-hash>
owner: extension
canonical_spec: <notion-page-id-or-doc-id>
```

Do not overburden authors with redundant metadata; keep fields that enable drift/version automation.

# Versioned documentation

During pre-1.0, distinguish `next`/development behavior from released behavior. Once compatibility matters, preserve release docs for stable versions instead of rewriting history in-place.

Version banners must clarify when a page describes development/main rather than installed release.

# Generated reference

Generate where practical:

- Action registry catalog;
- Property/Parameter schemas;
- Plugin permissions/capabilities;
- MCP method schemas;
- diagnostics codes;
- resource/token IDs;
- native-format schemas;
- Rust API links/rustdoc.

Generated files must say they are generated and identify the source generator.

# Code examples as tests

Copyable examples should compile/run in CI where practical. Plugin/MCP cookbook scripts become fixture tests. A documentation snippet that no longer works is a product regression.

# Diagrams

Prefer Mermaid/source-controlled diagrams for architecture/state machines. Images are acceptable for UI examples, but never make screenshots the only source of semantics. Images require alt text/caption and should be updateable without losing underlying rules.

# Search

Search must index both user guide and reference while preserving locale. Terminology aliases belong in metadata/glossary, not duplicated misleading headings.

# CI documentation gate

Pull requests affecting behavior/API/UI should run:

- VitePress build;
- broken-link/anchor check;
- Markdown lint/format policy;
- generated-reference drift check;
- code-snippet/example tests;
- English↔pt-BR coverage/freshness check;
- spelling/terminology checks where signal is reliable;
- site accessibility smoke test;
- duplicate semantic ID/link validation.

# Publishing

Recommended workflow:

1. merge to protected main/release branch;
2. CI builds VitePress deterministically;
3. publish static artifacts to GitHub Pages or selected static host;
4. deployment links to commit/version;
5. failed docs build blocks documentation deployment and, for contract changes, should block release.

# Site accessibility

The documentation site itself must support keyboard navigation, readable contrast, reduced motion, semantic headings, alt text, focus visibility and mobile reading even though Aubrieta desktop is not mobile-oriented.

# Translation workflow

English change → mark counterpart stale → translate pt-BR → review identifiers/normative wording → build both locales. Machine translation may produce a draft, but code agents must validate terminology and not translate API symbols.

# Documentation changelog

Public API/file-format/plugin/MCP changes appear in a compatibility/changelog section with migration guidance. Major user-workflow changes should update relevant tutorials, not only release notes.

# Offline/export consideration

Because VitePress emits static files, keep documentation mostly static and self-contained. Essential API reference should not require a live SaaS connection to read after the site is built.

# Verified VitePress capabilities — September 2026

The current VitePress documentation confirms built-in locale routing/configuration through `locales` and locale directory structure, and the default theme supports in-browser fuzzy local full-text search with locale-specific search translations. This fits Aubrieta's static bilingual documentation requirement without requiring a hosted search backend.

Official references:

- [VitePress — Internationalization](https://vitepress.dev/guide/i18n)
- [VitePress — Default Theme Search](https://vitepress.dev/reference/default-theme-search)

# VitePress boundary

VitePress is a **documentation renderer/site layer**, not a source of product truth or application runtime dependency. Repository Markdown/schemas/examples remain useful without VitePress, and the Aubrieta desktop application must not depend on the documentation site being online.

# Prumo/VitePress synchronization rule

Prumo may generate/organize documentation work items, indexes, microcontexts and implementation dossiers, but **VitePress repository Markdown remains the published implementation-facing documentation source**. Any Prumo-generated change intended to be stable must land as reviewable source-controlled files, pass the same EN→pt-BR and VitePress CI gates, and identify generated sections so handwritten rationale is not overwritten blindly.