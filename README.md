# Petunia Design Studio

Editor desktop open source de **vetor + raster + layout**, construído em Rust com arquitetura modular e edição não destrutiva.

A interface escolhida é **Qt/QML via CXX-Qt**. Core, Engine e Render permanecem independentes da UI.

## Workspace

```text
crates/
├── petunia-core    # documento, scene, paths, cor e invariantes
├── petunia-engine  # geometria, snapping, commands e avaliação
├── petunia-render  # rasterização, composição, caches e output
└── petunia-ui      # Qt/QML via CXX-Qt, ferramentas, workspace e interação
```

A direção arquitetural é:

```text
UI → Engine → Core
UI → Render → Core
```

Engine não depende de Render/UI. Render não depende de Engine/UI. Core não depende dos outros domínios Petunia.

## Stack base

- Rust
- Qt/QML via CXX-Qt — interface
- kurbo — curvas e geometria
- i_overlay — operações booleanas/topologia
- palette — matemática de cor
- rustybuzz — text shaping
- fontdue — rasterização de glifos
- image — codecs bitmap
- rayon — paralelismo de CPU
- serde — persistência
- uuid — identidade
- thiserror — erros tipados

A integração concreta de Qt/QML via `CXX-Qt` será adicionada ao `petunia-ui` conforme a implementação da interface avançar; a escolha arquitetural já está definida.

## Comandos

```bash
cargo check --workspace
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
```

## Documentação

A documentação técnica navegável vive em `website/`.

Para executar localmente:

```bash
python3 -m http.server 8080 -d website
```

Abra `http://localhost:8080`.

A documentação é a fonte canônica das decisões arquiteturais, filosofia, roadmap e matriz de implementação.
