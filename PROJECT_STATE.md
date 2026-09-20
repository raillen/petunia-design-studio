# Estado Atual do Projeto — Aubrieta Design

- Projeto: **Aubrieta Design**
- Data deste estado: **2026-09-19**
- Upstream normativo: [`docs/`](docs/) (Atlas Arquitetural e Decisões de Design)
- Governança de agentes: [`AGENTS.md`](AGENTS.md)
- Fase Ativa Atual: **P04 (Renderização GPU & Formatos de Intercâmbio Avançados)**
- Fases Concluídas: **P00 (Fundação)**, **P01 (Core Headless MVP)**, **P02 (Texto, Raster & SVG)**, **P03 (Extensão, Recursos e MCP)** — todas com suíte verde e gauntlet validado.

## Resumo Executivo

O núcleo *headless* do Aubrieta Design está estabelecido e estável:
1. Modelo canônico de documento com mutações exclusivas via `DocumentMutator` e `ChangeSet`.
2. Geometria 2D vetorial canônica com operações booleanas via adaptadores isolados.
3. Modelo de cor multivalorado (sRGB, CMYK, Lab, Spot) com adaptador defensivo `moxcms`.
4. Motor de texto com `TextStory`, style runs, quebra de linha e hit-testing.
5. Motor de rasterização esparso com tiles de 128×128 px, suporte a 8 e 16 bits por canal e pipeline de pincel.
6. Pacote nativo ZIP (`.aubrieta` / `.aubri`) com escrita atômica e exportador/importador SVG.
7. Recursos e Tokens (`aubrieta_resources`): tokens DTCG, temas Light/Dark, catálogo de strings en-US/pt-BR e icon mapping.
8. Serviços de Plataforma (`aubrieta_platform`): portas de clipboard, file dialogs e ambiente com adaptadores headless em memória.
9. Sistema de Extensões (`aubrieta_extension`): Plugin SDK, manifestos com permissões granulares, capability broker e sandbox Lua 5.4.
10. Servidor MCP (`aubrieta_mcp`): JSON-RPC 2.0 com descoberta, inspeção semântica, mutações transacionais e detecção de revisões defasadas.
11. Pipeline de validação `cargo xtask gauntlet` aprovando **80 testes** (62 unitários e 18 proptests), clippy limpo e checagem de arestas proibidas.

## Matriz de Implementação de Fases

| Fase | Título | Estado | Evidência Resumida | Próximo Passo |
|---|---|---|---|---|
| **P00** | Fundação e Endurecimento | `COMPLIANT` / `REVIEWING` | 18 testes, IDs monotônicos, `ChangeSet`, undo/redo, CI, audit limpo | Homologação formal dos goals |
| **P01** | Core Headless MVP | `COMPLIANT` / `REVIEWING` | 37 testes, geometria, cor, evaluation, scene, `.aubrieta` zip | Homologação formal dos goals |
| **P02** | Texto, Raster e SVG | `COMPLIANT` / `REVIEWING` | 46 testes, `aubrieta_text`, `aubrieta_raster` 128x128 16-bit, SVG I/O, CLI conformance | Homologação formal dos goals |
| **P03** | Extensão, Recursos e Plataforma | `COMPLIANT` / `REVIEWING` | 80 testes, `aubrieta_resources`, `aubrieta_platform`, `aubrieta_extension`, `aubrieta_mcp` | Homologação formal dos goals |
| **P04** | Renderização GPU & Formatos | `EXECUTING` | Especificações em `09.7`, `09.9`, `09.11` | `P04-G01` (`aubrieta_compositor`), `P04-G02` (`aubrieta_pdf`), `P04-G03` (`aubrieta_image_io`) |
| **P05** | Shell Desktop & Interatividade | `PLANNED` | Especificações em `09.24`, `09.27`, `10.x` | `AubrietaGuiBridge` e aplicação GPUI |
