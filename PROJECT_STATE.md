# Estado Atual do Projeto — Aubrieta Design

- Projeto: **Aubrieta Design**
- Data deste estado: **2026-09-19**
- Upstream normativo: [`docs/`](docs/) (Atlas Arquitetural e Decisões de Design)
- Governança de agentes: [`AGENTS.md`](AGENTS.md)
- Fase Ativa Atual: **P03 (Sistema de Extensão, Recursos e Plataforma)**
- Fases Concluídas: **P00 (Fundação)**, **P01 (Core Headless MVP)**, **P02 (Texto, Raster & SVG)** — todas com suíte verde e gauntlet validado.

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
9. Pipeline de validação `cargo xtask gauntlet` aprovando **67 testes** (53 unitários e 14 proptests), clippy limpo e checagem de arestas proibidas.

## Matriz de Implementação de Fases

| Fase | Título | Estado | Evidência Resumida | Próximo Passo |
|---|---|---|---|---|
| **P00** | Fundação e Endurecimento | `COMPLIANT` / `REVIEWING` | 18 testes, IDs monotônicos, `ChangeSet`, undo/redo, CI, audit limpo | Homologação formal dos goals |
| **P01** | Core Headless MVP | `COMPLIANT` / `REVIEWING` | 37 testes, geometria, cor, evaluation, scene, `.aubrieta` zip | Homologação formal dos goals |
| **P02** | Texto, Raster e SVG | `COMPLIANT` / `REVIEWING` | 46 testes, `aubrieta_text`, `aubrieta_raster` 128x128 16-bit, SVG I/O, CLI conformance | Homologação formal dos goals |
| **P03** | Extensão, Recursos e Plataforma | `EXECUTING` | 67 testes, `aubrieta_resources` e `aubrieta_platform` concluídos em `REVIEWING` | `P03-G03` (`aubrieta_extension`) e `P03-G04` (`aubrieta_mcp`) |
| **P04** | Renderização GPU & Formatos | `PLANNED` | Especificações em `09.7`, `09.9`, `09.11` | Integração `vello`/`wgpu`, PDF com `krilla` e `image_io` |
| **P05** | Shell Desktop & Interatividade | `PLANNED` | Especificações em `09.24`, `09.27`, `10.x` | `AubrietaGuiBridge` e aplicação GPUI |
