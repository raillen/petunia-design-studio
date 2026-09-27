# Getting started

Welcome to Petunia Design Studio. This guide takes you from zero to a working document in about 15 minutes.

```mermaid
flowchart LR
  A[Discover] --> B[Install]
  B --> C[Create document]
  C --> D[Draw & compose]
  D --> E[Proof & export]
  E --> F[Automate & extend]
  style B stroke-width:3px
  style C stroke-width:3px
```

## The 15-minute path

1. **[Installation](/getting-started/installation)** (~10 min) — install Rust, clone the repo, verify with `cargo run -p xtask -- verify`.
2. **[Quickstart](/getting-started/quickstart)** (~5 min) — create a surface, draw two shapes, combine them, undo, export SVG/PDF/PNG.

## What you need

- A 64-bit Linux, macOS or Windows machine with **Rust stable** (see [Installation](/getting-started/installation)).
- No GPU required: the headless CLI and `cargo test` suites run fully without a display.
- About 4 GB of free disk for the first workspace build.

## Where to go next

- Doing layout, illustration or photo work? Continue with the [Manual](/manual/).
- Want the full command reference? Open the [Tool catalog](/tools/).
- Automating or contributing? Head to [Developers](/developers/) and the [Constitution](/bible/).
