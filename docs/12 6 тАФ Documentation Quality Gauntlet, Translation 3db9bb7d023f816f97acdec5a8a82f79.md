# 12.6 — Documentation Quality Gauntlet, Translation Freshness, Accessibility & Release Gates

# Documentation gauntlet dimensions

Score and review:

- coverage;
- technical correctness;
- consistency with canonical decisions;
- clarity for newcomers;
- precision for implementers;
- code-agent retrievability;
- examples/tests;
- EN↔pt-BR freshness;
- accessibility;
- navigation/search;
- version/compatibility clarity.

# Coverage matrix

Every stable feature should map to:

- user-facing workflow docs if visible;
- architecture/implementation docs;
- API docs if externally accessible;
- troubleshooting/errors where meaningful;
- tests/fixtures;
- changelog/migration when compatibility changes.

# Broken-reference gate

Fail CI/release docs when canonical pages reference missing ActionId, TextId, IconId, PropertyId, method, schema anchor or file.

# Translation gate

For stable/release docs, pt-BR must not silently lag canonical English. Draft/next may permit temporary staleness only with visible marker and tracked completion before release.

# Readability review

Avoid giant walls of abstraction. Each implementation-heavy page should contain:

- purpose;
- mental model/diagram where useful;
- normative rules;
- examples;
- failure/edge cases;
- testing/DoD;
- cross-links.

# Accessibility gate for docs site

Keyboard navigation, heading hierarchy, visible focus, contrast, alt text, code-block usability, reduced motion and semantic landmarks are release requirements for the documentation website.

# Code-agent retrieval test

Periodically test questions such as:

- `How do I add a new dockable panel without coupling core to GPUI?`
- `Which permission does a plugin need to write a selected export file?`
- `How does MCP safely edit a stale document revision?`
- `Where do I register a new PropertyId?`
- `What must be updated when adding a new Action?`

A competent agent should retrieve one unambiguous canonical path without combining contradictory pages.

# Drift injection tests

Intentionally test CI against fixture drift: missing translation, removed action, changed schema, broken link, untested snippet and undocumented token. The documentation pipeline is not trusted until it catches expected failures.

# Release gate

A release with implementation changes cannot be declared documentation-complete until VitePress builds both locales, generated reference matches source, examples pass and compatibility changes have migration notes.

# Continuous audit

Run the documentation gap audit at milestone boundaries and after major architectural changes. Add new rows rather than assuming old completeness categories cover new subsystems.

# Documentation score evidence

Gauntlet scores must cite evidence. A `10/10` documentation score requires no known material gap for the milestone, successful EN/pt-BR build/freshness checks, valid generated references/examples, passing link/accessibility checks and a successful retrieval test against representative agent questions. Do not raise the score merely because prose became longer.

# Documentation performance/usability

Treat the site as a product surface: monitor build size/time, client search responsiveness on the generated corpus, navigation depth, broken-search/no-result cases and readability on narrow screens. Large generated references should be split/paginated logically so they remain usable by humans and retrievable by agents.