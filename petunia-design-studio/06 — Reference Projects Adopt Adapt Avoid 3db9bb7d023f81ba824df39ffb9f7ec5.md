# 06 — Reference Projects: Adopt / Adapt / Avoid

<aside>
🔬

**Authority:** this page is **Research / Prior Art**, not an implementation specification. It may justify or inspire ADRs, but code agents must not copy a reference project's architecture, dependency or UX behavior unless that choice is independently accepted in Aubrieta's canonical Atlas/ADR pages.

</aside>

# Regra

Referências são fontes de prior art. Aubrieta estuda implementações antes de decidir, mas não importa arquitetura por autoridade.

| Projeto | Adotar/estudar | Evitar copiar |
| --- | --- | --- |
| Graphite | nondestructive modifiers, central mutation API, derived metadata, stable IDs, memoization/invalidation, live boolean | documento inteiro node-first/JIT/CRDT prematuros |
| Affinity | Design↔Photo shared document, layers, clipping/masks, adjustments/live filters, personas, dense properties UX | replicar Publisher completo ou decisões fechadas de produto |
| Zed/GPUI | Actions, key contexts, command palette, focus, workspace density, high-performance editor architecture | acoplar Aubrieta ao modelo específico de text editor |
| Lapce/Floem | fine-grained reactive desktop UI, Rust-first application shell | vazar signal/UI state para Document Core |
| Rerun | render extraction, GPU resource caching, viewport architecture, MCP/inspection ideas | usar egui data model como domínio |
| Linebender | Kurbo/Vello/Parley/Fontique/Skrifa/AccessKit ecosystem coherence | assumir que UI experimental já resolve desktop tooling completo |
| Typst/Krilla | document/PDF rigor, font subsetting, test discipline | transformar Aubrieta em layout/typesetting engine |
| COSMIC Text | caret, selection, BiDi/IME interaction prior art | duplicar um segundo canonical text model |
| CorelDRAW | vector tool breadth, print-oriented workflows, variable data | legacy UI complexity |

# Graphite inheritance from VectorVonDoom study

Preservar explicitamente:

- `DocumentStore` + authoritative mutation layer;
- derived metadata/caches separados;
- live modifiers e destructive commit separados;
- parameter schemas reutilizáveis por Properties Panel, Plugins, MCP e futura Node View;
- Layers/Properties/Effect Stack como UX primária;
- eventual Node View apenas como outra apresentação das mesmas operations, não segunda verdade.

# Workspace/persona composition

Personas registram tools, panels, actions, shortcuts, overlays e inspectors. Usuário poderá futuramente salvar workspace layouts e misturar capacidades Design/Photo sem novos formatos de documento.