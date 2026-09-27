---
layout: home
title: Petunia Design Studio
titleTemplate: Vector + photo design, non-destructive by default
hero:
  name: Petunia Design Studio
  text: Design vector. Retouch photo. Break nothing.
  tagline: A desktop design studio with a non-destructive core, an open .ptnd format, and automation-first APIs (CLI, MCP, plugins).
  image:
    src: /logo.svg
    alt: Petunia Design Studio logo
  actions:
    - theme: brand
      text: Get started in 15 min
      link: /getting-started/quickstart
    - theme: alt
      text: Read the manual
      link: /manual/
    - theme: alt
      text: Tool catalog
      link: /tools/
features:
  - icon: 🖋️
    title: Design persona
    details: Pen, nodes, shapes, booleans, fills, gradients, typography and artboards over one shared document tree.
    link: /tools/
    linkText: Browse tools
  - icon: 📷
    title: Photo persona
    details: Selections, brushes, adjustments and masks as reorderable, non-destructive nodes — source pixels are never touched.
    link: /tools/
    linkText: Browse tools
  - icon: 🧬
    title: Non-destructive core
    details: Every mutation flows Action → Command → DocumentMutator → ChangeSet. Destructive ops (Bake, Expand, Rasterize) are explicit.
    link: /developers/architecture
    linkText: How it works
  - icon: 🤖
    title: Automation-first
    details: Headless CLI, JSON-RPC MCP server, Lua plugin sandbox and capability registries — no fake UI, ever.
    link: /developers/
    linkText: Developer docs
  - icon: 📦
    title: Open .ptnd format
    details: Atomic ZIP package with SVG/PDF/PNG export, preflight checks and CMYK soft-proofing for print.
    link: /manual/
    linkText: Learn the workflow
  - icon: 🌍
    title: Bilingual by construction
    details: English is canonical; pt-BR mirrors every page 1:1. New languages plug in through the same locale pattern.
    link: /contributing/translations
    linkText: Translate
---

## The journey

One path from first install to automated studio:

```mermaid
flowchart LR
  A[Discover] --> B[Install]
  B --> C[Create document]
  C --> D[Draw & compose]
  D --> E[Proof & export]
  E --> F[Automate & extend]
```

| Step | You do | Start here |
| ---- | ------ | ---------- |
| Discover | Understand what Petunia is and is not | [Constitution](/bible/) |
| Install | Build the workspace and launch the app or CLI | [Installation](/getting-started/installation) |
| Create document | Open your first `.ptnd` surface | [Quickstart](/getting-started/quickstart) |
| Draw & compose | Master tools, panels and personas | [Manual](/manual/) · [Tools](/tools/) |
| Proof & export | Preflight, soft-proof and ship SVG/PDF/PNG | [Manual](/manual/) |
| Automate & extend | Drive Petunia from scripts, agents and plugins | [Developers](/developers/) |

## Canonical vocabulary

Petunia uses precise names: a **Surface** hosts composition, a **Layer** is a role in the single document tree (not a parallel hierarchy), a **Binding** links a property to variable-data. New terms are never invented silently — see the [Glossary](/glossary) (EN ↔ PT-BR) and [SPEC-001](/bible/SPEC-001).
