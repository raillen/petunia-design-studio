# Installation

Install the Petunia Design Studio workspace from source. English is the canonical language of these instructions; the [pt-BR mirror](/pt/getting-started/installation) says exactly the same thing.

## 1. Prerequisites

| Requirement | Version | Check |
| ----------- | ------- | ----- |
| Rust toolchain | stable ≥ 1.80 | `rustc --version` |
| C build tools | gcc/clang + pkg-config | `cc --version` |
| Git | any recent | `git --version` |
| Node.js (docs only) | ≥ 18 | `node --version` |

::: tip Low-RAM machines
Linking the desktop app needs RAM. On machines with ≤ 8 GB, build test binaries single-threaded:

```bash
# Build with debug symbols off and one link job at a time
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test -p petunia-design -j 1
```

:::

## 2. Clone

```bash
# Clone the repository
git clone git@github.com:raillen/petunia-design-studio.git
cd petunia-design-studio
```

## 3. Verify the toolchain

```bash
# Fast gate: fmt check + clippy + tests + architecture boundary checks
cargo run -p xtask -- verify
```

A green `verify` means: formatting clean, zero clippy warnings, unit tests pass, and no domain crate imports GUI toolkit types.

## 4. Run the headless CLI

```bash
# End-to-end smoke: document -> geometry -> color -> evaluation -> save/reopen -> export summary
cargo run -p petunia-design-cli
```

## 5. Launch the desktop app

```bash
# Slint shell (primary UI)
cargo run -p petunia-design
```

## 6. Preview these docs locally (optional)

```bash
# Install the docs site dependencies and start the dev server
cd docs
npm install
npm run dev
```

Next: [Quickstart](/getting-started/quickstart) — your first document in 5 minutes.
