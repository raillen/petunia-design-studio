# 05 — GUI Technology Research & Decision Matrix

<aside>
🎯

**Decisão atual:** **GPUI é o alvo padrão e primário da interface do Aubrieta Design Suite.** O core continua GUI-agnóstico para preservar testabilidade, isolamento e capacidade de migração; isso não significa manter a escolha de shell indefinidamente aberta. Slint, Floem, Iced e egui permanecem como referências/contingência e não bloqueiam o desenvolvimento principal.

</aside>

# Critérios Aubrieta

Prioridade máxima: integração zero-copy/low-copy com wgpu/Vello; custom canvas; dense desktop tooling; docking/floating panels; IME/text; keyboard/actions; multi-window; drag/drop/clipboard; high-DPI; accessibility; file dialogs/native menus; theming; testability; Linux/Windows/macOS; estabilidade; facilidade para agentes de IA.

# Decisão atual — GPUI-first

**GPUI** é a shell primária de implementação. O desenvolvimento de interface deverá assumir GPUI/GPUI Kit por padrão e somente criar um adapter alternativo quando existir uma razão concreta de engenharia.

A hierarquia de adoção é:

1. `gpui-kit` como facade/version alignment;
2. `gpui-base` para comportamento e infraestrutura sem identidade visual imposta;
3. **Aubrieta Design System** como camada visual canônica;
4. `gpui-component` reutilizado seletivamente onde acelerar desenvolvimento sem limitar a identidade do produto;
5. bibliotecas auxiliares GPUI avaliadas por necessidade específica.

Slint, Floem, Iced e egui deixam de ser implementações obrigatórias equivalentes. Permanecem documentados como **fallbacks, benchmark arquitetural, fontes de prior art e contingência de longo prazo**. O experimento egui maximalista pode continuar como laboratório/devtool, mas não deve atrasar o caminho GPUI.

# Pesquisa ampliada / shortlist histórica

| Candidato | Fit preliminar | Leitura |
| --- | --- | --- |
| Slint | 9.4/10 | Top contender. DSL declarativa, APIs estáveis 1.x, bindings Rust/C++/JS/Python, tooling forte e API oficial experimental para integrar renderer externo wgpu. |
| Floem | 8.9/10 | Rust puro, fine-grained reactivity, bom alinhamento com Vello/Linebender; framework ainda amadurecendo/pre-1.0. |
| GPUI | 8.8/10 | Prior art Zed, GPU-first, excelente actions/key contexts; ainda pre-1.0 e muito ligado ao ritmo interno do Zed. |
| Iced | 8.4/10 | Cross-platform, modular, runtime renderer-agnostic, wgpu/tiny-skia, Elm model. Ainda se descreve como experimental. |
| Makepad | 8.0/10 | Plataforma Rust voltada explicitamente a creative software, DSL live-editable e GPU runtime próprio. Integração com nosso wgpu/compositor precisa prova. |
| Vizia | 7.9/10 | Declarativa, reactive, cross-platform, styling/hot reload; menor ecossistema. |
| Xilem/Masonry | 7.8/10 agora; alto potencial | Exatamente winit + Vello/wgpu + Parley/Fontique + AccessKit, mas oficialmente experimental. |
| Kael | experimental promissor | Novo framework GPU retained direcionado a editors/media/creative software; avaliar maturidade antes de qualquer compromisso. |
| Ribir | 7.2/10 | Data-centric reactive, Rust/WASM; interessante mas menor. |
| Blitz/Dioxus Native | experimental | HTML/CSS native renderer modular com Vello; bleeding edge, útil como pesquisa de futuro. |
| RFGUI | research only | Frame-graph retained GUI sobre wgpu; arquitetura interessante para compositor, widgets ainda em construção. |
| egui | 7.3/10 main UI; 9.5/10 tooling | Excelente para inspector/debug/dev tools e integração custom; immediate-mode e seus próprios non-goals reduzem preferência para shell final. |

# Alternative GUI references

Slint, Floem, Iced and egui remain documented because they provide useful prior art and a contingency path if GPUI later fails a critical product requirement. They should not be implemented in parallel with the primary GPUI shell by default.

- **Slint:** declarative UI separation/tooling reference and potential fallback shell.
- **Floem:** reactive Rust/Vello-aligned prior art.
- **Iced:** explicit state/message architecture and testability prior art.
- **egui:** maximalist experimental/devtool track and source of inspection/MCP/testing ideas.

The historical comparison table above is retained as research context, not the current selection ranking.

# Outras linguagens/shells — core continua Rust

| Shell | Viabilidade | Trade-off |
| --- | --- | --- |
| Qt Quick/QML + CXX-Qt | Muito alta | Shell desktop extremamente madura; CXX-Qt oferece Rust↔Qt/QML bidirecional. Adiciona Qt/C++ build/runtime/licensing e complexidade de integração. |
| TypeScript + Tauri | Alta para panels/chrome | Web UI muito produtiva e backend Rust natural. WebView↔native canvas/render integration é o ponto crítico; não duplicar o renderer em Canvas/WebGPU sem decisão explícita. |
| TypeScript + Electron | Tecnicamente alta | Ecossistema UI enorme e renderer Chromium consistente, mas footprint/runtime e bridge aumentam; baixa prioridade. |
| Python + PySide6/QML | Alta | Excelente para protótipo e Qt Quick maduro; runtime Python + bridge para Rust e packaging aumentam custo. Canvas nativo exige integração dedicada. |
| Python + DearPyGui | Média | Ótimo para devtools, inspector e protótipos GPU; menos adequado ao polish final da suíte. |
| Go + Wails v3 | Baixa como produto final | Excelente Go+WebView/TS, mas Aubrieta teria Rust→Go→WebView: camada extra sem benefício frente a Tauri. |
| Go + Fyne/Gio | Baixa-média | Native Go, mas integração FFI/cgo e renderer próprio criam segunda stack gráfica. |
| Kotlin + Compose Multiplatform | Média | Desktop GPU e UX declarativa maduros; JVM + FFI Rust e segunda toolchain aumentam custo. |
| Flutter/Dart | Média | Cross-platform e FFI C disponível; excelente UI, mas engine Flutter torna integração com nosso wgpu canvas mais arquiteturalmente pesada. |
| C# + Avalonia 12 | Média-alta | Desktop toolkit maduro e customizável; P/Invoke/NativeAOT bridge possível, mas adiciona .NET e repetiria parte do problema de múltiplas stacks. |

# GUI boundary remains mandatory

Even with GPUI selected, `AubrietaGuiBridge` remains coarse-grained. Never emit toolkit calls per path vertex, brush sample or render primitive. GPUI sends intents/actions and receives snapshots/deltas of application state. The creative canvas/render/compositor remains a Aubrieta engine concern, not a document-model concern and not a reason to let GPUI types enter domain crates.

# Protótipo e gauntlet obrigatórios — GPUI

Construir primeiro o mini-Aubrieta em **GPUI + GPUI Kit**, sobre o mesmo `AubrietaGuiBridge`, sem lógica de domínio exclusiva da GUI. O protótipo deve ter docking/panels, command palette, keyboard contexts, Layers tree com 10k+ rows, Properties, Design↔Photo switch, IME text, drag/drop, file dialog, accessibility semantics, high-DPI e canvas Aubrieta desacoplado.

Somente se um gate crítico falhar de forma estrutural deverá ser executado o mesmo benchmark em Slint/Floem/Iced/egui para comparação. A existência desses adapters continua sendo uma proteção arquitetural, não uma tarefa obrigatória do roadmap inicial.

## Experimento egui — regra especial

O protótipo egui deverá utilizar agressivamente o ecossistema auxiliar em vez de julgar `egui` isoladamente. Candidatos já confirmados para estudo incluem:

- `egui_tiles` e `egui_dock` para tiling/docking;
- `egui_ltreeview` para Layers/asset trees de alta escala;
- `egui_extras` para widgets/loaders auxiliares;
- `egui_palette` para command palette;
- `egui-command`/binding como prior art para command registry e atalhos;
- `egui_file_dialog` para file browser/dialog customizado;
- `egui-notify` ou `egui-toast` para notifications;
- `egui_plot` para histogramas, performance e debug visual;
- `egui_colorgradient` para gradient editing experimental;
- `egui-phosphor` e `egui-lucide` para iconografia;
- `catppuccin-egui` apenas como referência de theme plumbing, não identidade visual final;
- `egui_commonmark` para help/manual/dev documentation surfaces;
- `egui_code_editor` para scripts/devtools;
- `egui_inspection` + `egui_mcp` como referência prioritária para inspection tree, input injection, screenshots e testes por agentes.

A lista será expandida em pesquisa específica do ecossistema egui. O objetivo é descobrir se uma combinação cuidadosamente selecionada consegue superar as limitações percebidas de uma GUI immediate-mode em uma creative suite densa.

## Decision update — GPUI is primary

The Aubrieta Design Suite now standardizes on **GPUI as the primary/default UI target**. GPUI Kit is the preferred application infrastructure, with `gpui-base` as the main reusable behavior layer and a Aubrieta-owned design system on top. This is a product implementation decision, while the core remains GUI-agnostic as an architectural constraint.

Slint, Floem, Iced and egui remain valid fallback/reference paths. They are no longer peer candidates that must be developed in parallel.

### egui maximalist experiment baseline

The egui experiment should deliberately combine supporting crates instead of judging plain egui. Initial ecosystem candidates include `eframe`, `egui_tiles`/`egui_dock`, `egui_ltreeview`, `egui_extras`, `egui_palette`, `egui-command`, `egui_file_dialog`, `egui-notify`/toast helpers, `egui_plot`, gradient/color editing helpers, Lucide/Phosphor icon integrations, Markdown/code editor helpers, inspection/devtool crates and `egui_mcp`/semantic automation where compatible.

The goal is to answer a concrete question: **how far can an immediate-mode Rust UI be pushed toward an Affinity-class dense creative tool when the ecosystem is used aggressively rather than judging the base crate alone?**

### GPUI UI Gauntlet

The primary gauntlet targets **GPUI + GPUI Kit + Aubrieta Design System**: Design/Photo switch, tool rail, command palette, context toolbar, canvas host, Layers tree with 10k+ rows, Properties, Color, resizable/dockable panels, keyboard contexts, IME text editing, drag/drop, file dialogs, HiDPI, accessibility semantics, theming and automation/test hooks.

Evaluation must include visual polish, customization, dense-editor ergonomics, docking/workspaces, large-tree performance, input/IME, accessibility, agent testing, runtime performance, cross-platform behavior and development ergonomics. Alternative GUI gauntlets are conditional fallback investigations only.

## GPUI primary-stack documentation

A dedicated child page now defines the canonical GPUI layers, support libraries, design-system rules, testing strategy and icon architecture. 3D/OpenGL/WGPUI research has been removed from the Aubrieta scope because it belonged to another project context.

[05.1 — GPUI Primary UI Stack, Ecosystem & Icons](05%201%20%E2%80%94%20GPUI%20Primary%20UI%20Stack,%20Ecosystem%20&%20Icons%203db9bb7d023f812ca59cc6d12cb13c67.md)