# Estado Atual do Projeto — Aubrieta Design

- Projeto: **Aubrieta Design**
- Data deste estado: **2026-09-20**
- Upstream normativo: [`docs/`](docs/) (Atlas Arquitetural e Decisões de Design)
- Governança de agentes: [`AGENTS.md`](AGENTS.md)
- Fase Ativa Atual: **P05 (Shell Desktop & Interatividade) — Concluída em `REVIEWING`**
- Fases Concluídas: **P00 (Fundação)**, **P01 (Core Headless MVP)**, **P02 (Texto, Raster & SVG)**, **P03 (Extensão, Recursos e MCP)**, **P04 (Renderização, PDF, Raster I/O e Prova de Cor)**, **P05 (Shell Desktop & Interatividade)** — todas com suíte verde e gauntlet validado.

## Resumo Executivo

O núcleo completo, o subsistema de renderização/I/O e a camada de interface desktop do Aubrieta Design estão estabelecidos e estáveis:
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
11. Motor de Composição e Renderização (`aubrieta_render`): contrato de 16 blend modes (W3C/PDF), grupos de isolamento, planejador de superfícies offscreen e renderizador de pixels determinístico para RGBA8 e RGBA16.
12. Exportação Vetorial PDF (`aubrieta_io::pdf`): exportador profissional com `krilla`, suporte sRGB e CMYK nativos, caminhos vetoriais e análise de degradação/preflight.
13. Importação/Exportação Raster (`aubrieta_io::image_io`): adaptadores para PNG, JPEG, WebP e TIFF com preservação de 8-bit e 16-bit e sniffing defensivo.
14. Gestão de Cor e Soft-Proofing (`aubrieta_color::proof`): simulação de perfis de prensa (SWOP/FOGRA), detecção de cores fora de gama (out-of-gamut) e políticas de preservação numérica de CMYK.
15. Ponte de Apresentação e Interatividade (`aubrieta_ui_gpui`):
    - `AubrietaGuiBridge`: facade unificada implementando portas semânticas (`ActionQueryPort`, `CommandPort`, `PropertyPort`, `DocumentQueryPort`, `SelectionPort`, `InspectionPort`) com isolamento estrito de tipos da GUI do domínio.
    - Viewport & Canvas: `ViewportCamera` com zoom infinito invariante centrado no cursor (0.1% a 25600%) e `SnapEngine` com histerese anti-jitter e guias visuais.
    - Máquina de Ferramentas: ferramentas interativas `SelectTool` (seleção, marquee, translação, duplicate-drag Alt/Option), `PenTool` (tangentes Bézier e fechamento de curva), `NodeTool` e `ShapeTool` (retângulo, elipse, polígono com restrição 1:1 via Shift).
    - Painéis Reativos: `LayersPanelController` (visibilidade, lock, reordenação), `PropertiesPanelController` (fill, stroke, opacity, bounds) e `HistoryPanelController` (inspeção de undo/redo).
    - Shell Desktop & Harness Headless: `AubrietaShell` e `MockGuiAdapter` com 8 invariantes de conformidade em CI headless.
16. Pipeline de validação `cargo xtask gauntlet` aprovando **130 testes** (100 unitários e de integração + 30 proptests), clippy sem advertências e sem arestas proibidas de arquitetura.

## Matriz de Implementação de Fases

| Fase | Título | Estado | Evidência Resumida | Próximo Passo |
|---|---|---|---|---|
| **P00** | Fundação e Endurecimento | `COMPLIANT` / `REVIEWING` | 18 testes, IDs monotônicos, `ChangeSet`, undo/redo, CI, audit limpo | Homologação formal dos goals |
| **P01** | Core Headless MVP | `COMPLIANT` / `REVIEWING` | 37 testes, geometria, cor, evaluation, scene, `.aubrieta` zip | Homologação formal dos goals |
| **P02** | Texto, Raster e SVG | `COMPLIANT` / `REVIEWING` | 46 testes, `aubrieta_text`, `aubrieta_raster` 128x128 16-bit, SVG I/O, CLI conformance | Homologação formal dos goals |
| **P03** | Extensão, Recursos e Plataforma | `COMPLIANT` / `REVIEWING` | 80 testes, `aubrieta_resources`, `aubrieta_platform`, `aubrieta_extension`, `aubrieta_mcp` | Homologação formal dos goals |
| **P04** | Renderização GPU & Formatos | `COMPLIANT` / `REVIEWING` | 110 testes, `aubrieta_render`, `aubrieta_io::pdf`, `aubrieta_io::image_io`, `aubrieta_color::proof` | Homologação formal dos goals |
| **P05** | Shell Desktop & Interatividade | `COMPLIANT` / `REVIEWING` | 130 testes, `aubrieta_ui_gpui` (bridge, viewport, snapping, select/pen/node/shape tools, painéis e mock adapter) | Próximas fases / homologação |

