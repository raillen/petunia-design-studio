# Petunia Design Studio

Editor desktop open source de **vetor + raster + layout**, construído em Rust com arquitetura modular e edição não destrutiva.

A interface escolhida é **Qt/QML via CXX-Qt**. Core, Engine e Render permanecem independentes da UI.

## Workspace

```text
crates/
├── petunia-core    # documento, scene, paths, cor e invariantes
├── petunia-engine  # geometria, snapping, commands e avaliação
├── petunia-render-model # snapshots e recursos derivados imutáveis
├── petunia-render  # rasterização, composição e output
├── petunia-ui      # sessão e interação headless
└── petunia-desktop # janela Qt/QML e bridge CXX-Qt opcional
```

A direção arquitetural alvo é:

```text
UI → Engine → Core
UI → Render → Core

Engine ─→ petunia-render-model ←─ Render
```

`petunia-render-model` contém o contrato imutável compartilhado entre Engine e Render.

Engine não depende de Render/UI. Render não depende de Engine/UI. Core não depende dos outros domínios Petunia.

## Stack base

- Rust
- Qt/QML via CXX-Qt — interface
- kurbo — curvas e geometria
- i_overlay — operações booleanas/topologia
- palette — matemática de cor
- Little CMS 2 (binding lcms2, backend estático) — transformações ICC
- wasmi — execução WASM com limites
- qrcodegen e barcoders — geração de códigos
- rustybuzz — text shaping
- fontdue — rasterização de glifos
- image — codecs bitmap
- rayon — paralelismo de CPU
- serde — persistência
- uuid — identidade
- thiserror — erros tipados

A integração Qt/QML via `CXX-Qt` vive em `petunia-desktop`. A janela utiliza a sessão e as transações existentes, com personas configuráveis, painéis e ícones Phosphor/Tabler embarcados.

O toolchain validado está fixado em `rust-toolchain.toml`. O build headless não exige Qt; o backend estático de ICC exige um compilador C.

## Comandos

```bash
cargo check --workspace --all-targets --locked
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Para executar a GUI, instale o SDK Qt 6.8 (Core, Gui, Qml, Quick, Quick Controls,
Dialogs e plugin SVG), um compilador C++ e um linker LLD, Gold ou Mold:

```bash
cargo run -p petunia-desktop --features native --bin petunia-studio
```

`./scripts/run-studio.sh` também detecta o SDK local de desenvolvimento, quando presente.

Se houver vários SDKs Qt, defina `QMAKE` com o caminho do qmake6 escolhido.
O guia de execução e os limites atuais estão em [GUI nativa](website/docs/04-ui/native-desktop.md).

## Documentação

A documentação técnica navegável vive em `website/`.

Para executar localmente:

```bash
python3 -m http.server 8080 -d website
```

Abra `http://localhost:8080`.

A documentação é a fonte canônica das decisões arquiteturais, filosofia, roadmap e matriz de implementação.


## Code agents e workforce Prumo

Antes de implementar, consultar [Diretivas para Code Agents](website/docs/07-agents/index.md), [website/llms.txt](website/llms.txt) e [workforce completa](website/docs/07-agents/workforce-catalog.md). O projeto usa um subconjunto local do Prumo, e as listas do site incluem também os agentes/skills/recipes complementares upstream, sem afirmar que foram instalados.

O código Python/C++/QML anterior foi preservado no [commit 13fe6b4](https://github.com/raillen/petunia-ds/tree/13fe6b408752b244ee56d4765ea13435eb3ef921); [política de migração/reuso](website/docs/07-agents/legacy-migration.md). Preservar testes e componentes de valor quando compatíveis, sem mudar a arquitetura Rust atual.
