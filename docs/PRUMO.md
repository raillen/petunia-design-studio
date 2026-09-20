# Prumo — Aubrieta Design (Intent Router)

Roteador central de intenção e mapa canônico de navegação do projeto **Aubrieta Design** para humanos e agentes de IA.

---

## 0. Autoridade Documental e Regras do Projeto

- [`AGENTS.md`](../AGENTS.md) — Regras invioláveis de arquitetura (fronteiras de crates, ausência de vazamento de GUI para o domínio, contratos de mutação via `DocumentMutator` e `ChangeSet`).
- [`PROJECT_STATE.md`](../PROJECT_STATE.md) — Estado operacional atual do projeto, fases e matriz de conformidade.
- [`prumo.json`](../prumo.json) — Manifesto do projeto e configuração do Prumo CLI.
- [`docs/`](.) — Atlas de arquitetura e especificações normativas de produto.

---

## 1. Mapa de Fases do Projeto

1. **Fase P00 — Fundação e Endurecimento** (Concluída)
   - Contratos de IDs estáveis, diagnósticos estruturados, modelo de documento e mutator, commands com undo/redo, capability registry.
2. **Fase P01 — Core Headless MVP** (Concluída)
   - Motores de geometria (`aubrieta_geometry`), cor multivalorada (`aubrieta_color`), grafo de avaliação (`aubrieta_evaluation`), cena de render (`aubrieta_render`) e I/O nativo ZIP (`aubrieta_io`).
3. **Fase P02 — Texto, Rasterização e SVG** (Concluída)
   - Motor de texto (`aubrieta_text`), motor de rasterização em tiles 128x128 8/16-bit (`aubrieta_raster`), exportador/importador SVG (`aubrieta_io`) e conformidade CLI (`aubrieta-cli`).
4. **Fase P03 — Recursos, Plataforma, Extensão e MCP** (Em Execução)
   - `aubrieta_resources` (Design tokens, i18n en-US e pt-BR).
   - `aubrieta_platform` (Portas de serviços do sistema operacional).
   - `aubrieta_extension` (Host de plugins seguro com runtime Lua 5.5 / WASM).
   - `aubrieta_mcp` (Servidor de inspeção e mutação por agentes de IA).
5. **Fase P04 — Renderização GPU e Formatos Avançados** (Planejada)
   - Compositor GPU `vello` / `wgpu`, exportador PDF (`krilla`), importador/exportador de imagens raster.
6. **Fase P05 — Shell Interativa Desktop e Ferramentas** (Planejada)
   - Bridge desacoplada `AubrietaGuiBridge` e aplicação interativa GPUI.

---

## 2. Comandos e Verificação

- `cargo xtask verify`: fmt, clippy, testes e verificação de arestas de dependência arquitetural.
- `cargo xtask gauntlet`: bateria completa de conformidade do projeto.
- `cargo run -p aubrieta-cli`: execução do pipeline headless de ponta a ponta.
