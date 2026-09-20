# 00 — Product Charter & Scope

<aside>
🧭

**Aubrieta Design** não é três aplicações coladas. É um único documento criativo híbrido, apresentado por duas Personas: **Design** e **Photo**.

</aside>

# Produto

Objetivo: construir uma suíte desktop de design profissional com edição vetorial e bitmap integradas, layout leve, dados variáveis, gerenciamento de cor profissional e automação profunda, mantendo baixo acoplamento entre domínio, renderer e interface.

# Personas

## Design

Responsável por desenho vetorial, paths, shapes, tipografia, layout leve, symbols/assets, perspectiva, efeitos, artboards/surfaces e data merge.

## Photo

Responsável por PixelLayer, pintura, seleções raster, masks, adjustment layers, filtros, crop, retoque e operações bitmap.

Trocar Persona altera ferramentas, painéis, context toolbar e actions expostas; **não altera o documento nem converte objetos**.

# Recursos de publishing incorporados ao Design

- Surface/Page/Artboard unificada;
- margins, columns, guides e baseline grid;
- bleed;
- text frames e text flow;
- paragraph/character/object styles;
- page/surface numbering quando necessário;
- Data Merge / Variable Data;
- geração em lote de surfaces, assets e PDFs.

Não fazem parte do objetivo inicial: book management, long-document orchestration, índices complexos, notas de rodapé avançadas, imposição editorial completa ou um Publisher Studio separado.

# Escopo de cor e saída

- documento semanticamente RGB/CMYK/Gray/Lab/Spot/Registration;
- ICC profiles e soft-proof planejados como capacidade central;
- PDF profissional via backend dedicado;
- PDF/X-1a fora do caminho crítico inicial, podendo ser convertido/validado externamente;
- SVG e raster exports como adapters.

# Referências de produto

- Affinity Designer/Photo: UX, hierarchy, masks, adjustments, live filters, personas e fluxo não destrutivo;
- CorelDRAW: ferramentas vetoriais, produção gráfica e variable-data workflows;
- Graphite: prior art de engine não destrutiva e separação frontend/backend;
- Zed/Lapce: densidade, command palette, actions/keymaps e arquitetura desktop moderna.

# Intended users and workflows

Aubrieta V1 is designed for illustrators, graphic designers, small studios, print-oriented creators, photographers/image editors and advanced hobbyists who need a single vector+raster document without the complexity of a full publishing suite. Code agents must optimize for **real desktop creative workflows**, not generic CRUD/application patterns.

Representative V1 jobs:

- create/edit a vector illustration and export SVG/PDF/raster;
- combine vector, text and raster objects in one document;
- edit a photo non-destructively and reuse it inside a Design composition;
- create multi-Surface print/social assets with shared styles/symbols;
- prepare color-managed professional output;
- generate record-driven assets through Data Merge;
- automate repeatable operations through Actions, plugins or MCP.

# V1 product outcome

V1 is successful when the common Design and Photo workflows are **coherent, reversible, color-managed, performant and scriptable** using the same canonical document. Breadth must not be obtained by shipping disconnected half-implementations. A smaller feature with complete persistence, undo, accessibility, automation and error recovery is preferred over a larger feature that bypasses canonical contracts.

# Scope-status rule

This page owns product scope, while detailed requirement status follows section 12.8. Every feature named here or in Persona pages must be classifiable as `V1 Required`, `Milestone Required`, `Post-V1 Candidate`, `Research / Prior Art`, `Open ADR`, `Historical` or `Out of Scope`. Bare words such as “future”, “later” or “planned” are not sufficient scope states for code agents.

# Product-level non-goals for V1

- no separate Publisher persona or long-book/document-management system;
- no collaborative cloud document architecture/CRDT requirement;
- no mandatory account/login/cloud service;
- no AI generation dependency for core editing workflows;
- no node-first canonical document representation;
- no WebView-based creative canvas;
- no plugin or MCP mutation path that bypasses Actions/Commands/validation;
- no hard-coded English-only UI or fixed icon/theme identity;
- no GUI toolkit dependency in canonical document/engine crates.

# Product quality constraints

A V1 workflow is incomplete if it lacks applicable undo/redo, save/reopen fidelity, error recovery, accessibility semantics, tokenized/localizable presentation, performance/resource budget, deterministic automation equivalent and documentation. These are product requirements, not post-feature polish.