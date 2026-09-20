# P01 Dossier — MVP Headless Core (P01-G01 a P01-G05)

## Status / scope

- P01-G01 `REVIEWING` (Geometria MVP — paths, afins, booleanos i_overlay/kurbo)
- P01-G02 `REVIEWING` (Cor MVP — sRGB, CMYK, Lab, moxcms adapter, display convert)
- P01-G03 `REVIEWING` (Evaluation & Render MVP — cache por geração, extração de cena rebuildable)
- P01-G04 `REVIEWING` (I/O Nativo MVP — pacote `.aubrieta` / `.aubri`, manifest.json, escrita atômica)
- P01-G05 `REVIEWING` (CLI Conformance MVP — pipeline headless end-to-end sem dependências de GUI)
- Escopo: `V1_REQUIRED` (fatia mínima headless funcional dos motores do core).

## Goal e outcomes

1. **Geometria (`aubrieta_geometry`)**: Tipos canônicos sem vazamento de bibliotecas externas na API pública (`GPoint`, `GRect`, `GPath`, `GAffine`, `BooleanOp`). Adaptador bidirecional para `kurbo` e `i_overlay` para união e interseção poligonal com precisão.
2. **Cor (`aubrieta_color`)**: Modelo de cor desacoplado (`ColorValue`: `Srgb`, `Cmyk`, `Lab`, `Named`, `Spot`, `Registration`). Conversão direta sRGB com clamping defensivo. Conversão Lab→sRGB via D50/D65 illuminants. Adaptador `moxcms` isolado para conferência de ponto branco. Suporte a perfis ICC marcado explicitamente como `POST_V1` com mensagem clara em vez de fallback silencioso.
3. **Evaluation (`aubrieta_evaluation`)**: Grafo de derivação leve com invalidação por geração (`generation`) e cache memoizado de sumário de superfícies e objetos.
4. **Render (`aubrieta_render`)**: Extração de cena (`Scene`, `SceneFragment`) determinística e desacoplada da GUI; backend headless de referência para testes e conformidade (`HeadlessSummaryBackend`).
5. **I/O Nativo (`aubrieta_io`)**: Formato de pacote zip aberto com `manifest.json`, versão de schema `NATIVE_SCHEMA_VERSION = 1`, documento em `document/document.json`, suporte a extensões `.aubrieta` e `.aubri`, e gravação atômica via arquivo temporário irmão (`.tmp.<name>`) seguido de `rename`.
6. **CLI Conformance (`aubrieta-cli`)**: Executável de ponta a ponta sem GUI: criação de documento → adição de superfície e objeto → aplicação de preenchimento semântico → cálculo geométrico e booleano → conversão de cor Lab/sRGB → invalidação de cache e undo/redo via `History` → extração e renderização da cena → salvamento atômico em pacote nativo e reabertura com verificação de igualdade estrutural.

## Non-goals nesta fase

- Compositor gráfico GPU (Vello / wgpu) — reservado para onda com dependências nativas pesadas.
- Registro completo de perfis ICC e LittleCMS harness diferencial (`POST_V1`).
- Motor de texto e modelagem de parágrafos/shaping (Parley) (`POST_V1`).
- Cache de rasterização por ladrilhos (tiles 8/16-bit) (`POST_V1`).
- Shell interativa e componentes GPUI (`POST_V1`).

## Referências canônicas

- `00 — Product Charter & Scope`
- `04 — Core Stack Decisions` (Rust workspace, kurbo, i_overlay, moxcms)
- `07 — Quality Gates & Architecture Guardrails`
- `09.21 — Architecture Governance` (fronteiras invioláveis e ausência de ciclos)
- `09.22 — Cargo Workspace & Crate Topology` (domínio nunca importa GUI/GPU)
- `10.3 — Shapes & Boolean Operations`
- `10.7 — Surfaces, Artboards, Pages`
- `12.10 — Prumo CLI Workflow, Goals/Waves, Implementation Dossiers`
- `14.1 / 14.8 — Test Architecture & xtask Conformance Gauntlet`

## Mapa de crates (Workspace ampliado P01)

- `aubrieta_foundation`: IDs tipados (`ObjectId`, `SurfaceId`), diagnósticos, gerador monotônico, schema version.
- `aubrieta_document`: Modelo de documento, `DocumentMutator`, `ChangeSet`, serialização de formato interno.
- `aubrieta_application`: Ações (`Action`), Comandos (`Command`), histórico (`History`) com undo/redo defensivo, registro de capacidades (`CapabilityRegistry`).
- `aubrieta_jobs`: Token e rastreamento de cancelamento (`CancellationToken`).
- `aubrieta_geometry`: Geometria 2D, primitivas, curvas Bezier, retângulos de delimitação, transformações afins, operações booleanas poligonais.
- `aubrieta_color`: Modelo de cor multivalorado, conversão de espaço de cores, landmarks Lab, facade de transformação.
- `aubrieta_evaluation`: Rastreador de invalidação de dados derivados por geração.
- `aubrieta_render`: Extração de cena e backend headless de referência.
- `aubrieta_io`: Leitura e gravação de arquivos nativos `.aubrieta` / `.aubri` baseados em ZIP com escrita atômica.
- `apps/aubrieta-cli`: Executável de teste de conformidade de fluxo total.
- `xtask`: Verificador de arquitetura (arestas proibidas), `test`, `conformance`, `fmt`, `clippy` e `gauntlet`.

## Evidência de verificação P01

- `cargo test --workspace`: 37 passed (29 testes unitários + 8 testes de propriedade proptest), 0 failed.
- `cargo clippy --workspace --all-targets -- -D warnings`: limpo (0 avisos).
- `cargo fmt --all --check`: limpo.
- `cargo run -p aubrieta-cli`: execução com sucesso (pipeline completo validado).
- `cargo xtask gauntlet`: verde (verifica formatação, clippy, testes unitários e de propriedade, arestas arquiteturais proibidas, conformidade de CLI, presença de fixtures e checagem de docs).
- `cargo audit`: 0 vulnerabilidades em 133 dependências escaneadas no `Cargo.lock`.

## Handoff e Próximos Passos

1. Revisão humana dos Goals de P00 e P01 em `REVIEWING`.
2. Próxima fase planejada (P02):
   - Motor de texto (`aubrieta_text`) com shaping e métricas.
   - Motor de raster/tiles (`aubrieta_raster`).
   - Infraestrutura de exportação e conversores adicionais (SVG / PDF).
   - Shell inicial GPUI ou integração de plugins WASM/Lua.
