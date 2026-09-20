# Aubrieta Design

<aside>
🌺

**Status:** **Aubrieta Design** é o nome canônico selecionado para a aplicação Design + Photo. `.petunia` permanece reservado ao Petunia3D. A família mantém a política de dar a cada grande aplicativo um nome botânico próprio. Para Aubrieta Design, `.aubrieta` é a extensão nativa canônica e `.aubri` é um alias curto compatível do mesmo formato; `.abrt` é rejeitada por colisão de nomenclatura com o ecossistema ABRT do Fedora/RHEL. O caderno anterior VectorVonDoom permanece como referência histórica e fonte de decisões arquiteturais úteis.

</aside>

# Visão do produto

**Aubrieta Design** será uma aplicação criativa desktop, Rust-first, com um único documento híbrido e duas personas principais no mesmo workspace:

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

**Fechado:** nome **Aubrieta Design**; identidade Design + Photo; Document Core agnóstico à GUI; Surface; Actions/Commands; IDs estáveis; arquitetura não destrutiva; engines vetor/raster separados; semantic color e automação unificada; formato nativo aberto com `.aubrieta` canônica e `.aubri` como alias curto compatível, pacote ZIP aberto com payload estrutural JSON + recursos binários + projeções SVG/PDF; namespace semântico built-in `aubrieta.*`; bridge GUI canônica `AubrietaGuiBridge`; `moxcms` primário com LittleCMS fallback/oracle; tiles raster 128×128 por padrão; plugin UI declarativa; Unicode MessageFormat 2; Appearance Stack multi-fill/multi-stroke; reusable layout via Symbols/SurfaceTemplate; PDF/X nativo adiado para pós-V1.

**Candidatos fortes:** Vello, wgpu, Kurbo, iOverlay, Parley stack, moxcms, usvg/resvg, Krilla, AccessKit e Wasmtime para o tier de plugins altamente isolado. **Plugin scripting runtime V1:** **Lua 5.5 + `mlua` aceito**; o gauntlet comparativo continua como evidência/conformance e gatilho para ADR futuro, não como bloqueio da decisão.

**Interface primária:** **GPUI** é o alvo padrão e primário de desenvolvimento da shell desktop. A implementação deve usar o ecossistema GPUI Kit como base de infraestrutura, priorizando `gpui-base` para comportamento sem estilo e o **Aubrieta Design System** como identidade visual própria; `gpui-component` pode ser reutilizado seletivamente para acelerar partes maduras da interface. A shell conversa com o core pela `AubrietaGuiBridge`. Slint, Floem, Iced e egui permanecem documentados como alternativas, benchmarks e contingência arquitetural, não como candidatas equivalentes no fluxo principal. O core continua GUI-agnóstico para impedir acoplamento estrutural ao toolkit.

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

[08 — Interface Atlas & Aubrieta Design System](08%20%E2%80%94%20Interface%20Atlas%20&%20Aubrieta%20Design%20System%203db9bb7d023f816898b8cdc5efef3c76.md)

# Documentation depth rule

The Interface Atlas is now the required depth benchmark for **all** Aubrieta subsystems. Architecture, engines, persistence, plugins, IO, color, typography, raster/vector behavior, automation, security and release engineering must each document contracts, internal model, lifecycle, failure modes, performance, extension points, tests and implementation sequence rather than only high-level intent.

<aside>
🧩

**New architectural invariant:** Aubrieta is modular not only at crate boundaries but at runtime composition boundaries. Built-in and external capabilities register through semantic contracts; tools, panels and subsystems must be attachable/detachable without turning neighboring features into implicit dependencies.

</aside>

The detailed contracts for this invariant are maintained in the new **Architecture & Implementation Atlas** and **Functional Engine Atlas**.

[08 — Interface Atlas & Aubrieta Design System](08%20%E2%80%94%20Interface%20Atlas%20&%20Aubrieta%20Design%20System%203db9bb7d023f816898b8cdc5efef3c76.md) remains the UX specification; the new architecture/functionality atlases will use the same no-gap discipline for implementation internals.

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

[11 — Naming & Brand Audit](11%20%E2%80%94%20Naming%20&%20Brand%20Audit%203db9bb7d023f8101bbf2f52337cbde26.md)

[12 — Code Agent Implementation Handbook & Living Documentation](12%20%E2%80%94%20Code%20Agent%20Implementation%20Handbook%20&%20Living%20D%203db9bb7d023f81049058df8d98185faa.md)

[13 — Full Notebook Page-by-Page Audit & Conformance Ledger](13%20%E2%80%94%20Full%20Notebook%20Page-by-Page%20Audit%20&%20Conformanc%203db9bb7d023f810fb6fdf8aa804ef74e.md)

[14 — Implementation Evidence, Verification & AgentOps Atlas](14%20%E2%80%94%20Implementation%20Evidence,%20Verification%20&%20Agent%203df9bb7d023f81a78feedf6410de5065.md)