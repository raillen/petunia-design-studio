# Instalação

Instala o workspace do Petunia Design Studio a partir do fonte. O inglês é o idioma canônico destas instruções; este espelho pt-BR diz exatamente a mesma coisa.

## 1. Pré-requisitos

| Requisito | Versão | Verificação |
| --------- | ------ | ----------- |
| Toolchain Rust | stable ≥ 1.80 | `rustc --version` |
| Ferramentas de compilação C | gcc/clang + pkg-config | `cc --version` |
| Git | qualquer recente | `git --version` |
| Node.js (só docs) | ≥ 18 | `node --version` |

::: tip Máquinas com pouca RAM
Linkar o app desktop exige RAM. Em máquinas com ≤ 8 GB, compile os binários de teste em uma thread:

```bash
# Compila com símbolos de debug desligados e um link por vez
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test -p petunia-design -j 1
```

:::

## 2. Clonar

```bash
# Clona o repositório
git clone git@github.com:raillen/petunia-design-studio.git
cd petunia-design-studio
```

## 3. Verificar a toolchain

```bash
# Portão rápido: fmt check + clippy + testes + verificações de fronteira da arquitetura
cargo run -p xtask -- verify
```

Um `verify` verde significa: formatação limpa, zero avisos do clippy, testes passando e nenhum crate de domínio importando tipos de toolkit GUI.

## 4. Rodar a CLI headless

```bash
# Smoke de ponta a ponta: documento -> geometria -> cor -> avaliação -> salvar/reabrir -> resumo de exportação
cargo run -p petunia-design-cli
```

## 5. Abrir o app desktop

```bash
# Shell Slint (UI primária)
cargo run -p petunia-design
```

## 6. Previsualizar esta documentação localmente (opcional)

```bash
# Instala as dependências do site e inicia o servidor de desenvolvimento
cd docs
npm install
npm run dev
```

Próximo: [Início rápido](/pt/getting-started/quickstart) — seu primeiro documento em 5 minutos.
