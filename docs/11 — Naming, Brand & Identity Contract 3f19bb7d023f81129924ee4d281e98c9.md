# 11 — Naming, Brand & Identity Contract

# Canonical identity

Produto: **Petunia Design Studio**.

Extensão nativa: **.PTND**.

Namespace semântico built-in: **ptnd.**.

# Product vocabulary

- Persona: Design, Photo;
- Surface: abstração canônica para artboard/page/export region;
- Studio Panel: painel dockável;
- Workspace: layout de UI;
- Action: intenção invocável;
- Command: mutação validada;
- Tool: interaction state machine;
- Live Effect/Adjustment: transformação não destrutiva.

# Technical naming

Python packages usam snake_case; Python classes PascalCase; ActionId e PropertyId são namespaced strings estáveis; C++ namespaces curtos e orientados ao domínio. Evitar nomes genéricos como Manager, Helper, Utils, Data quando um termo de domínio existe.

# Legacy

Aubrieta e decisões Rust/Slint são históricas. Leitores/migrations podem reconhecer formatos antigos se fixtures existirem, mas novos nomes e APIs não perpetuam namespace legado.

# UI language

Textos visíveis são recursos localizáveis; ActionId, ToolId, PanelId, PropertyId e schemas nunca usam label localizada como identidade.

[11.1 — Canonical Product Vocabulary, Semantic IDs & Forbidden Ambiguity](11%201%20%E2%80%94%20Canonical%20Product%20Vocabulary,%20Semantic%20IDs%20%203f19bb7d023f8140a267fb87715e391f.md)

[11.2 — Brand Personality, Visual Identity Principles & Non-Copying Rules](11%202%20%E2%80%94%20Brand%20Personality,%20Visual%20Identity%20Principl%203f19bb7d023f81be9af8d6123c8d61de.md)

[11.3 — Namespace, File Extensions, MIME Types, URIs & External Identifiers](11%203%20%E2%80%94%20Namespace,%20File%20Extensions,%20MIME%20Types,%20URI%203f19bb7d023f818ba386f382fe9b9d86.md)

[11.4 — Localization, Text IDs, Terminology Glossary & Translation Governance](11%204%20%E2%80%94%20Localization,%20Text%20IDs,%20Terminology%20Glossar%203f19bb7d023f81398869ec8d91d62d3b.md)

[11.5 — Iconography, Cursor, Tool Naming & Action Naming Contract](11%205%20%E2%80%94%20Iconography,%20Cursor,%20Tool%20Naming%20&%20Action%20N%203f19bb7d023f81f59b64cd46ab2298f0.md)

[11.1 — Stable Identifier Taxonomy & Namespace Rules](11%201%20%E2%80%94%20Stable%20Identifier%20Taxonomy%20&%20Namespace%20Rule%203f19bb7d023f813cb3b2e47f972af1d5.md)

[11.2 — User-Facing Terminology, Localization Keys & Consistency Glossary](11%202%20%E2%80%94%20User-Facing%20Terminology,%20Localization%20Keys%20%203f19bb7d023f818a80e9d671f0537692.md)

[11.3 — File Extensions, MIME Types, Clipboard/MIME & Protocol Names](11%203%20%E2%80%94%20File%20Extensions,%20MIME%20Types,%20Clipboard%20MIME%203f19bb7d023f8153b730eba52853171a.md)

[11.4 — Brand/UI Identity Separation from Affinity & Reference Products](11%204%20%E2%80%94%20Brand%20UI%20Identity%20Separation%20from%20Affinity%20%203f19bb7d023f81b58625fddcc9f843dd.md)