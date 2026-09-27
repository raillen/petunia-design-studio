# Petunia Design Studio

<aside>
🌺

**Superseding identity decision — 2026-09-21:** o produto anteriormente chamado **Aubrieta Design** passa a chamar-se **Petunia Design Studio**. A extensão nativa canônica passa a ser **.PTND**. O app grava .PTND por padrão; leitura pode aceitar variação de caixa apenas por tolerância de plataforma, sem criar outro formato. Aubrieta Design, .aubrieta e .aubri tornam-se **identidade histórica/deprecated**, válidos apenas em migração, import de projetos antigos e documentação histórica explicitamente marcada. A GUI primária passa a ser **Slint**, preservando o core Rust e a fronteira UI-agnostic.

</aside>

# Visão do produto

**Petunia Design Studio** será uma aplicação criativa desktop, Rust-first, com um único documento híbrido e duas personas principais no mesmo workspace:

- **Design** — edição vetorial profissional, tipografia, composição, layout leve, surfaces/artboards, recursos selecionados de publishing, data merge e dados variáveis.
- **Photo** — edição bitmap, pintura, seleção, máscaras, adjustments, filtros e retoque, compartilhando o mesmo documento, layers, color engine, history e exportação da Persona Design.

Não haverá uma Persona Publisher separada. Recursos editoriais de maior valor para design — margins, columns, guides, baseline grid, text frames/flow, bleed e variable-data workflows — pertencem ao Design.

# Herança deliberada do VectorVonDoom

Este caderno usa [VectorVonDoom — Engineering Notebook](https://app.notion.com/p/VectorVonDoom-Engineering-Notebook-3d69bb7d023f812689e0d88726a3b927?pvs=21) como fonte histórica, preservando e adaptando principalmente:

- **Document Core independente de GUI/renderer** e dependências externas atrás de adapters;
- **Surface unificada** para artboard/page/export region conforme contexto;
- uma única API autoritativa de mutação via **Actions → Commands → DocumentMutator**;
- IDs estáveis e referências por handle/ID, nunca identidade baseada em posição na memória;
- estado canônico separado de **DocumentDerivedData**, SelectionState, ViewportState e RenderCache;
- edição não destrutiva, EffectChain tipada e operações destrutivas apenas quando explícitas;
- live boolean + bake/expand explícito;
- perspectiva unificada para vetor, texto e raster, separando ProjectiveTransform, PerspectivePlane/Grid e Warp/Envelope;
- UI fina: personas são composições de tools/panels/actions/capabilities, não donas do documento;
- arquitetura de testes, fuzzing, performance budgets e gates de qualidade como parte do design do sistema;
- Graphite como prior art de arquitetura, sem transformar o documento inteiro em um node graph.

# Princípios não negociáveis

1. **Document is truth; renderer is projection.**
2. **GUI é um adapter substituível.** Nenhum crate de domínio conhece Floem, Slint, GPUI, Qt, WebView, TypeScript, Python ou Go.
3. **Color is semantic.** RGB de monitor é uma representação; CMYK/Lab/Gray/Spot permanecem semanticamente no documento.
4. **Non-destructive by default.** Rasterize, Expand, Bake e Convert to Curves são operações explícitas.
5. **Every user operation is an Action/Command.** UI, atalhos, command palette, plugins e MCP usam o mesmo caminho.
6. **Derived state is disposable.** Caches, hit targets, thumbnails e render resources podem ser reconstruídos.
7. **Vector e raster convivem no mesmo scene/document model.** Trocar Persona não converte o documento.
8. **Rust puro quando suficientemente bom; FFI somente por vantagem excepcional e comprovada.**
9. **Modularidade é invariante de produto.** Tools, panels, effects, importers/exporters, data sources e subsistemas contribuem por capabilities/registries estáveis e devem poder ser habilitados, desabilitados ou substituídos sem dependências laterais implícitas.
10. **UI e recursos são semanticamente tokenizados.** Produção referencia `TextId`, `IconId`, `SettingId` e design tokens; strings, paths de ícones e cores literais não são identidade de feature.
11. **GPLv3 é a licença canônica do projeto.** Licenças e obrigações de dependências/assets permanecem auditadas no pipeline de release.

# Escopo técnico atual

- Rust como linguagem do core e engines.
- Vector Engine e Raster Engine separados, convergindo em compositor compartilhado.
- CMYK/ICC completo permanece requisito do documento e exportação; PDF/X-1a fica fora do caminho crítico inicial e poderá ser convertido/validado externamente.
- PDF profissional, SVG e raster exports são adapters; nenhum formato externo define o modelo nativo.
- Plugin SDK runtime-neutral e capability-based como capacidade de primeira classe; **Lua 5.5 via `mlua` é o runtime de scripting V1 aceito**. Wasmtime/WASI permanece como tier preferencial de componentes de alta isolação. Python/JavaScript permanecem linguagens de automação externa via MCP/client SDKs e só entram como runtimes embedded adicionais por novo ADR. Automação/MCP usa os mesmos contratos semânticos.

# Estado das decisões

**Fechado:** nome **Petunia Design Studio**; identidade Design + Photo; Document Core agnóstico à GUI; Surface; Actions/Commands; IDs estáveis; arquitetura não destrutiva; engines vetor/raster separados; semantic color e automação unificada; formato nativo aberto com **`.PTND`** canônica, pacote ZIP aberto com payload estrutural JSON + recursos binários + projeções SVG/PDF; namespace semântico built-in **`ptnd.*`**; bridge GUI canônica **`PetuniaDesignGuiBridge`**; **Slint** como shell primária; `moxcms` primário com LittleCMS fallback/oracle; tiles raster 128×128 por padrão; plugin UI declarativa; Unicode MessageFormat 2; Appearance Stack multi-fill/multi-stroke; reusable layout via Symbols/SurfaceTemplate; PDF/X nativo adiado para pós-V1. `.aubrieta`/`.aubri` são apenas compatibilidade legada durante a migração.

**Candidatos fortes:** Vello, wgpu, Kurbo, iOverlay, Parley stack, moxcms, usvg/resvg, Krilla, AccessKit e Wasmtime para o tier de plugins altamente isolado. **Plugin scripting runtime V1:** **Lua 5.5 + `mlua` aceito**; o gauntlet comparativo continua como evidência/conformance e gatilho para ADR futuro, não como bloqueio da decisão.

**Interface primária:** **Slint** é o alvo padrão e primário da shell desktop. A interface usa o **Petunia Design System** e conversa com o core exclusivamente pela **`PetuniaDesignGuiBridge`**. `slint-viewer`/LSP podem apoiar authoring e inspeção; Lucide/Tabler entram por `IconId` sem se tornar identidade de domínio; diálogos nativos ficam atrás de platform adapters. GPUI, Floem, Iced e egui permanecem apenas como prior art, benchmarks ou contingência arquitetural. O core continua GUI-agnóstico para impedir acoplamento estrutural ao toolkit.

# Fonte histórica

[VectorVonDoom — Engineering Notebook](https://app.notion.com/p/VectorVonDoom-Engineering-Notebook-3d69bb7d023f812689e0d88726a3b927?pvs=21)

[00 — Product Charter & Scope](00%20%E2%80%94%20Product%20Charter%20&%20Scope%203db9bb7d023f8179b359f64100d8ed45.md)

[01 — Architecture, GUI Boundary & Canonical Document](01%20%E2%80%94%20Architecture,%20GUI%20Boundary%20&%20Canonical%20Docume%203db9bb7d023f81c8a92be6b662aa25d3.md)

[02 — Design Persona: Vector, Layout & Variable Data](02%20%E2%80%94%20Design%20Persona%20Vector,%20Layout%20&%20Variable%20Data%203db9bb7d023f81efbeb5d915be067e6f.md)

[03 — Photo Persona: Raster, Masks & Nondestructive Imaging](03%20%E2%80%94%20Photo%20Persona%20Raster,%20Masks%20&%20Nondestructive%20%203db9bb7d023f81aca90ce2a485809264.md)

[04 — Canonical Rust Stack & Engine Boundaries](04%20%E2%80%94%20Canonical%20Rust%20Stack%20&%20Engine%20Boundaries%203db9bb7d023f813e826bd9ead40363b8.md)

[05 — GUI Technology Research & Decision Matrix](05%20%E2%80%94%20GUI%20Technology%20Research%20&%20Decision%20Matrix%203db9bb7d023f81159d32e9bab39c9383.md)

[06 — Reference Projects: Adopt / Adapt / Avoid](06%20%E2%80%94%20Reference%20Projects%20Adopt%20Adapt%20Avoid%203db9bb7d023f81ba824df39ffb9f7ec5.md)

[07 — Quality, Gauntlets, Plugins, MCP & AI Development](07%20%E2%80%94%20Quality,%20Gauntlets,%20Plugins,%20MCP%20&%20AI%20Develop%203db9bb7d023f8148ab13e85800df4af7.md)

[08 — Interface Atlas & Petunia Design System](08%20%E2%80%94%20Interface%20Atlas%20&%20Petunia%20Design%20System%203db9bb7d023f816898b8cdc5efef3c76.md)

# Documentation depth rule

The Interface Atlas is now the required depth benchmark for **all** Petunia Design Studio subsystems. Architecture, engines, persistence, plugins, IO, color, typography, raster/vector behavior, automation, security and release engineering must each document contracts, internal model, lifecycle, failure modes, performance, extension points, tests and implementation sequence rather than only high-level intent.

<aside>
🧩

**New architectural invariant:** Petunia Design Studio is modular not only at crate boundaries but at runtime composition boundaries. Built-in and external capabilities register through semantic contracts; tools, panels and subsystems must be attachable/detachable without turning neighboring features into implicit dependencies.

</aside>

The detailed contracts for this invariant are maintained in the new **Architecture & Implementation Atlas** and **Functional Engine Atlas**.

[08 — Interface Atlas & Petunia Design System](08%20%E2%80%94%20Interface%20Atlas%20&%20Petunia%20Design%20System%203db9bb7d023f816898b8cdc5efef3c76.md) remains the UX specification; the new architecture/functionality atlases will use the same no-gap discipline for implementation internals.

<aside>
📚

Documentation rule: each non-UI subsystem must reach the same implementation-level completeness as the Interface Atlas before it is considered architecturally frozen. Repository/VitePress documentation is authored canonically in English (`en-US`) with mandatory Brazilian Portuguese (`pt-BR`) translation for release, following section 12.

</aside>

<aside>
🤖

**Code-agent rule:** implementation agents must build a small authoritative microcontext, preserve modular/core↔UI boundaries, execute fixed gauntlets and update English + pt-BR living documentation/VitePress in the same change. They are executors of documented product contracts, not a new source of undocumented architecture.

</aside>

<aside>
🧱

Current documentation state: the earlier architecture-depth gap has been remediated through sections 09–13. The current frontier is implementation evidence: deterministic conformance, fixtures, fuzzing, performance, detach proof, UI/UX semantic goldens and reproducible release evidence, canonicalized in section 14.

</aside>

# Documentation expansion

Architecture and functional subsystems now have dedicated implementation atlases so future agents cannot treat the high-level pages as sufficient implementation specifications. The audit is a living acceptance criterion and must be repeated at milestone boundaries.

---

[09 — Architecture & Implementation Atlas](09%20%E2%80%94%20Architecture%20&%20Implementation%20Atlas%203db9bb7d023f810991aff0b9836407e0.md)

[10 — Functional Engine Atlas](10%20%E2%80%94%20Functional%20Engine%20Atlas%203db9bb7d023f81a2b96dc9446aca0a77.md)

[11 — Naming, Brand & Identity Migration Audit](11%20%E2%80%94%20Naming,%20Brand%20&%20Identity%20Migration%20Audit%203db9bb7d023f8101bbf2f52337cbde26.md)

[12 — Code Agent Implementation Handbook & Living Documentation](12%20%E2%80%94%20Code%20Agent%20Implementation%20Handbook%20&%20Living%20D%203db9bb7d023f81049058df8d98185faa.md)

[13 — Full Notebook Page-by-Page Audit & Conformance Ledger](13%20%E2%80%94%20Full%20Notebook%20Page-by-Page%20Audit%20&%20Conformanc%203db9bb7d023f810fb6fdf8aa804ef74e.md)

[14 — Implementation Evidence, Verification & AgentOps Atlas](14%20%E2%80%94%20Implementation%20Evidence,%20Verification%20&%20Agent%203df9bb7d023f81a78feedf6410de5065.md)

[15 — Petunia Design Studio Rebrand, Slint Migration & Total Assurance Program](15%20%E2%80%94%20Petunia%20Design%20Studio%20Rebrand,%20Slint%20Migratio%203e29bb7d023f8105b05de5d6e6c83ade.md)

[15.A — Identity Namespace, .PTND Format, Naming Migration & Legacy Compatibility](15%20A%20%E2%80%94%20Identity%20Namespace,%20PTND%20Format,%20Naming%20Mig%203e29bb7d023f816c91d5c2c78c82bad4.md)

[15.B — Slint Primary UI Architecture, Component Boundaries & Shell Contract](15%20B%20%E2%80%94%20Slint%20Primary%20UI%20Architecture,%20Component%20Bo%203e29bb7d023f816db021f7e14acfee45.md)

[15.C — Total Assurance Adoption: Surface Coverage, Evidence, Blind Spots & 10/10 Gates](15%20C%20%E2%80%94%20Total%20Assurance%20Adoption%20Surface%20Coverage,%20%203e29bb7d023f81e4a3bdcc43da794f59.md)

[15.D — Performance, Security, Reliability & Long-Session Constitution](15%20D%20%E2%80%94%20Performance,%20Security,%20Reliability%20&%20Long-S%203e29bb7d023f8179a446f5ad776380d9.md)

[15.E — Documentation Refactor Ledger, Authority Migration & No-Gap Checklist](15%20E%20%E2%80%94%20Documentation%20Refactor%20Ledger,%20Authority%20Mi%203e29bb7d023f81d7b6f9ef79fe634443.md)