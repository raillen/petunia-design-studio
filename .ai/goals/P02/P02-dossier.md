# P02 Dossier — Motores de Texto, Rasterização e I/O Vetorial SVG (P02-G01 a P02-G04)

## Status / scope

- P02-G01 `REVIEWING` (Motor de Texto e Tipografia — TextStory, runs, offsets, layout facade, hit testing)
- P02-G02 `REVIEWING` (Motor de Rasterização e Tiles — sparse 128x128 tiles, 8/16-bit depths, brush pipeline, blend modes)
- P02-G03 `REVIEWING` (Exportador Vetorial SVG — exportação/importação de documento e caminhos vetoriais)
- P02-G04 `REVIEWING` (Integração CLI e Gauntlet P02 — fluxo headless estendido em aubrieta-cli e checagem de arquitetura)
- Escopo: `V1_REQUIRED` (núcleos semânticos de texto, raster e intercâmbio vetorial).

## Goal e outcomes

1. **Texto e Tipografia (`aubrieta_text`)**:
   - `TextStoryId`: Identificador tipado e estável emitido por `IdGenerator`.
   - `TextOffset`: Offset de bytes em UTF-8 com validação estrita de fronteiras escalares de caracteres Unicode.
   - `CharacterStyle` / `CharacterStyleRun`: Propriedades de fonte, tamanho, peso, itálico, espaçamento e cor semântica.
   - `ParagraphStyle` / `ParagraphStyleRun`: Alinhamento (`Left`, `Center`, `Right`, `Justify`), altura de linha, recuo e espaçamento de parágrafo.
   - `TextStory`: Separação canônica entre conteúdo/estilo e contêineres de apresentação. Suporte a inserção e remoção com reajuste automático de style runs.
   - `TextLayout`: Facade de layout com quebra de linhas automática por largura máxima, cálculo de bounding box, baseline e hit testing de coordenadas.

2. **Rasterização e Tiles (`aubrieta_raster`)**:
   - Resolução canônica de tiles CPU de **128×128 px** conforme `09.6`.
   - `BitDepth`: Suporte nativo a profundidades de 8 bits e 16 bits por canal como requisito de arquitetura V1.
   - `PixelFormat`: `Rgba8`, `Rgba16`, `Gray8`, `Gray16`.
   - `TileCoord`: Coordenada discreta em espaço de camada com suporte a coordenadas negativas.
   - `TileMap`: Armazenamento esparso onde tiles ausentes são transparentes e não consomem memória.
   - `BrushDab` e pipeline de pincel: Suporte a carimbos pontuais com raio, dureza, opacidade, cor e modos de mesclagem (`Normal`, `Multiply`, `Screen`).

3. **Intercâmbio Vetorial SVG (`aubrieta_io`)**:
   - `export_document_svg`: Emissão de envelope SVG estruturado contendo superfícies e objetos do documento.
   - `export_path_d` e `parse_path_d`: Conversão bidirecional entre `GPath` e comandos SVG (`M`, `L`, `Q`, `C`, `Z`) com roundtrip verificado.

4. **Conformidade E2E e Gauntlet (`apps/aubrieta-cli` & `xtask`)**:
   - Fluxo completo sem GUI integrado em `aubrieta-cli`: Documento → Geometria → Cor → Avaliação → Cena → Pacote Nativo (.aubrieta/.aubri) → Reabertura → Texto/Tipografia → Raster/Tiles 16-bit → Exportação SVG.
   - Regras de arestas proibidas em `xtask` estendidas para `aubrieta_text` e `aubrieta_raster` (isolamento rigoroso de GPUI, Vello, wgpu, Krilla, mlua).

## Referências canônicas

- `00 — Product Charter & Scope`
- `04 — Canonical Rust Stack & Engine Boundaries`
- `09.6 — Raster Engine: Tiles, Brushes, Masks, Selections & Pixel Storage`
- `09.8 — Typography, Text Editing & Layout Engine`
- `09.11 — Import Export Adapter Contracts`
- `09.22 — Cargo Workspace, Crate Topology, Dependency Direction`
- `12.10 — Prumo CLI Workflow, Goals/Waves, Implementation Dossiers`
- `14.8 — xtask Verification and Gauntlet Loops`

## Mapa de crates (Workspace ampliado P02)

- `aubrieta_foundation`: IDs tipados (`ObjectId`, `SurfaceId`, `TextStoryId`), diagnósticos, gerador monotônico, schema version.
- `aubrieta_document`: Modelo de documento, `DocumentMutator`, `ChangeSet`.
- `aubrieta_application`: Ações, Comandos, histórico com undo/redo, registro de capacidades.
- `aubrieta_jobs`: Token de cancelamento.
- `aubrieta_geometry`: Primitivas 2D, caminhos afins, booleanos poligonais.
- `aubrieta_color`: Modelo de cor multivalorado, conversão de espaço de cores, landmarks Lab.
- `aubrieta_text`: Modelo de TextStory, style runs, facade de layout e hit-testing.
- `aubrieta_raster`: Tiles esparsos 128x128, profundidades 8/16-bit, pipeline de dabs de pincel e blend modes.
- `aubrieta_evaluation`: Cache derivado por geração.
- `aubrieta_render`: Extração de cena e backend headless de referência.
- `aubrieta_io`: Formato nativo ZIP (`.aubrieta` / `.aubri`) e exportador/importador SVG.
- `apps/aubrieta-cli`: Runner E2E de conformidade do pipeline completo.
- `xtask`: Verificador de arquitetura, testes e gauntlet.

## Evidência de verificação P02

- `cargo test --workspace`: **46 passed** (38 unitários + 8 proptests), 0 failed.
- `cargo clippy --workspace --all-targets -- -D warnings`: **0 warnings** (limpo).
- `cargo fmt --all --check`: **100% formatado**.
- `cargo run -p aubrieta-cli`: execução com sucesso (pipeline total validado).
- `cargo xtask gauntlet`: **P00/P01/P02 slice green** (verificação de fmt, clippy, testes, conformidade, arquitetura sem arestas proibidas, fixtures e docs).
- `cargo audit`: **0 vulnerabilidades** em 135 dependências escaneadas no `Cargo.lock`.

## Handoff e Próximos Passos (Fase P03)

1. Revisão humana dos Goals de P02 em `REVIEWING`.
2. Fase P03 — Sistema de Extensão, Recursos e Plataforma:
   - `aubrieta_resources`: Pacotes de recursos, tokens e internacionalização (`i18n`).
   - `aubrieta_extension`: API neutra de plugins e runtime de scripting (`mlua` / Lua 5.4+).
   - `aubrieta_platform`: Portas de serviços do SO (sistema de arquivos, clipboard, janelas).
