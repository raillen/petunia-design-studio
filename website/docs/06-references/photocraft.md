# PhotoCraft — auditoria técnica aplicada ao Petunia

**Snapshot:** [storytold/photocraft@5ecc860a](https://github.com/storytold/photocraft/tree/5ecc860ac1abf8fa7e3f218c0474cdcce49fa6c7) em 2026-10-08. Auditado por código e documentação; **sem execução ou validação de fidelidade**. O README declara alpha, portanto o nome de uma ferramenta não implica qualidade pronta para produção.

## 1. Arquitetura confirmada

Workspace Rust 2024 versão 0.5.0, 23 crates e apps desktop/CLI/web. Usa `egui/eframe 0.36`, `rayon`, `serde`, `arc-swap`, `bytemuck`, `image`; GUI é cliente do Engine. [`crates/engine/src/lib.rs`](https://github.com/storytold/photocraft/blob/5ecc860ac1abf8fa7e3f218c0474cdcce49fa6c7/crates/engine/src/lib.rs) descreve `Session::execute` como entrypoint único de command ID + params JSON; UI, CLI, controle e MCP chegam pelo mesmo dispatcher.

Camadas/famílias da árvore:
- Base: `geom`, `color`, `cms`, `raster`, `doc`.
- Operações: `ops`, `vector`, `paint`, `algo`, `text`.
- Imagem: `compose` (CPU), `gpu` (wgpu), `codecs`, `raw`, `psd`.
- Fluxos: `engine`, `io`, `format`, `automation`, `plugins`, `tablet`, `ui-egui`.

[`docs/architecture.md`](https://github.com/storytold/photocraft/blob/5ecc860ac1abf8fa7e3f218c0474cdcce49fa6c7/docs/architecture.md) descreve engine-first, dados puros, comandos estáveis, overlays como dados, schemas de parâmetros e COW para snapshots. **Caveat documental:** o cabeçalho dessa página se declara draft de 2026-09-30 e ainda discute possível UI alternativa; `Cargo.toml`, `AGENTS.md` e código corrente confirmam `egui`. Não tomar um design draft antigo como implementação vigente sem comparar o source.

## 2. `photocraft-raster`: tiles esparsos COW

[`crates/raster/src/lib.rs`](https://github.com/storytold/photocraft/blob/5ecc860ac1abf8fa7e3f218c0474cdcce49fa6c7/crates/raster/src/lib.rs) documenta `Surface` como plano lógico de tiles 256×256, `BTreeMap<TileCoord, Arc<Tile>>`, `PixelFormat` e `default_pixel`: tiles inexistentes leem transparência/default em vez de alocar a tela inteira. Clonagem compartilha `Arc` e apenas tiles sujos são copiados. `Surface` usa formato de pixel explícito e suporta masks com default não transparente.

**Alta aderência:** Petunia já especificou Tiles/COW/DirtySets em [Brush + Raster](#/docs/02-engine/brush-raster.md). Examinar estratégia de dirty tile invalidation, copy-on-write e snapshots, mas não copiar `Surface` como tipo PTND sem adaptar formato de cores, precision, tile layout, defaults e UUID das camadas.

**Contrato proposto de adapter:** `PixelLayerTileStore` tipado, `TileCoord`, `TileReadSnapshot`, `TileWriteTransaction`, `DirtyTileSet`, com conversão no boundary Render/IO. Testar layer sparse, huge/negative coordinates, mask white-default, memory budget e rollback durante stroke.

## 3. Composição: CPU como oráculo, GPU por passes

[`crates/compose/src/lib.rs`](https://github.com/storytold/photocraft/blob/5ecc860ac1abf8fa7e3f218c0474cdcce49fa6c7/crates/compose/src/lib.rs) contém compositing CPU RGBA f32 com blend modes, opacity × fill opacity, visibility, masks/density, clipping groups, passthrough versus isolated, adjustment layers e fill layers.

[`crates/gpu/src/lib.rs`](https://github.com/storytold/photocraft/blob/5ecc860ac1abf8fa7e3f218c0474cdcce49fa6c7/crates/gpu/src/lib.rs) descreve GPU compositor wgpu: residency por pages, uploads apenas de tiles alterados, planner de passes, effect maps cached, dirty region de effects e referência de equivalência CPU/GPU ~1/255 segundo comentário upstream.

**Excelente padrão de engenharia:** usar CPU oracle determinístico e testar parity com GPU em corpus multi-layer: blends, clipping, opacity, effects, color, alpha, nested groups, partial dirty regions. Não significa que GPU PhotoCraft possa ser transplantada diretamente para o `RenderSnapshot` Petunia.

**Incompatibilidade importante:** o comentário da composição informa que o modelo de referência então compõe em **display RGB**, incluindo caminho CMYK convertido. Petunia já define direção de composição linear/premultiplied e Color Management própria. Não trocar sem análise o modelo de mistura de cor para "seguir Photoshop"; oferecer modo de compatibilidade separado se exigido por roundtrips PSD, com decisão arquitetural explícita.

## 4. Documento: layering, efeitos e tipos autorais

[`crates/doc/src/lib.rs`](https://github.com/storytold/photocraft/blob/5ecc860ac1abf8fa7e3f218c0474cdcce49fa6c7/crates/doc/src/lib.rs) estabelece `children[0]` como layer inferior (ordem compositing; UI inverte a lista), tipos para layers não destrutivas: adjustment, fill, text, shape, smart object; paint e gradients, layer effects, comps, variables, vector path, text.

Pistas de módulos:
- `crates/doc/src/adjust.rs` — schema de adjustment.
- `crates/doc/src/effects.rs` e `compose/src/effects.rs` — strokes, glow, shadow, bevel e afins.
- `crates/compose/src/masks.rs` e `gradient_fill.rs` — masks/fills.
- `crates/doc/src/vector.rs` e `crates/vector/src/` — shapes/paths integrados a layer tree.
- `crates/engine/src/layer_*_cmds.rs`, `adjust_cmds.rs`, `smart_cmds.rs` — comandos de edição.

**Adaptar sem misturar modelos:** mapear semanticamente `PixelLayer`, `AdjustmentObject`, `Mask`, `TextObject`, `VectorPath` e seu `Appearance` dentro do Core Petunia, sem duplicar uma segunda árvore que possua IDs, nesting, clipping e Undo concorrentes.

## 5. Paint engine e presets

[`crates/paint/src/brush.rs`](https://github.com/storytold/photocraft/blob/5ecc860ac1abf8fa7e3f218c0474cdcce49fa6c7/crates/paint/src/brush.rs): brush data serializável e componentes de dinâmica opcionais (pressure, tilt, wheel, fade, scattering, tip shapes etc.). O source define limites `MAX_BRUSH_SIZE` e `MAX_SCATTER` para controlar custo/overflow de dabs. `crates/paint/src/dynamics.rs`, `render.rs`, `procedural.rs`, `retouch.rs`, `mixer.rs` completam pipeline.

[`crates/paint/src/tile.rs`](https://github.com/storytold/photocraft/blob/5ecc860ac1abf8fa7e3f218c0474cdcce49fa6c7/crates/paint/src/tile.rs) define `GrayTile` 16-bit serializado compactamente para tips/textures. Nota de robustez: `GrayTile::from_f32` possui um `assert_eq!` de tamanho; na integração Petunia **não** chamar esta API com dimensões/slices derivadas de entrada não confiável sem validação prévia. Não extrapolar isso para afirmar falha geral do projeto.

**Reuso recomendado:** esquema de presets/dynamics, dabs resamplados por arc length e incremental tile painting, test fixtures. Integrar com nosso `StrokeSample` (pointer pressure/tilt/rotation/time), `OneEuro`, Paint Engine e `BrushPreset`, preservando `flow` versus `opacity`, e evitando converter sample/stroke em Command por frame.

**A11y:** Pressure é opcional, e cada dinâmica deve possuir alternativa numérica/key input. Cursors/redimensionamento de brush não podem depender de Alt exclusivo, nem capturar teclado de campos.

## 6. Vector dentro do raster: útil, mas não autoridade

[`crates/vector/src/edit.rs`](https://github.com/storytold/photocraft/blob/5ecc860ac1abf8fa7e3f218c0474cdcce49fa6c7/crates/vector/src/edit.rs): move anchors, arrasta direction handles, reformata segmento e converte smooth/corner, com `Hit::Anchor([usize;2])`, `Hit::Handle([usize;2], side)`, `Hit::Segment([usize;2], t)`.

**Diferença semântica:** aqui knots são endereçados pelo par de índices `[subpath,knot]` validado; Petunia exige `NodeId` e `ContourId` estáveis. A rotina geométrica de preview/Commit pode inspirar nossa Engine, mas índices não podem virar referência durável em PTND ou comandos de agentes.

**Outra diferença:** path editing é complementar à edição raster no PhotoCraft; não usar sua implementação para substituir o modelo autoral vetorial mais rico do Petunia (Symmetric, LiveBuild, multi-path selection, provenance).

## 7. PSD/PSB e formatos de documento

[`crates/psd/src/`](https://github.com/storytold/photocraft/tree/5ecc860ac1abf8fa7e3f218c0474cdcce49fa6c7/crates/psd/src) contém parse/write modules `file`, `layer`, `descriptor`, `tagged`, `compression`, `resources`, `patterns`, `path` etc. README anuncia roundtrip visual de corpus psd-tools e preservation de blocos desconhecidos; números de fidelity são **declarações upstream, não nossos testes**.

[`crates/format/src/lib.rs`](https://github.com/storytold/photocraft/blob/5ecc860ac1abf8fa7e3f218c0474cdcce49fa6c7/crates/format/src/lib.rs) descreve `.pcraft` como ZIP ou pasta com `manifest.json`, tiles/blobs content-addressed via hash + compressão, previews opcionais, save incremental e recovery.

**Oportunidades separadas:**
1. Tratar `photocraft-psd` como referência potencial para I/O PSD/PSB por adapter; testes de roundtrip devem ser independentes e respeitar licenças.
2. Reaproveitar ideias de persistência incremental/content-addressing e atomic-write, sem trocar PTND por `.pcraft`.
3. Preservar raw PSD blocks com provenance e compatibilidade, mas nunca serializá-los no Core sem modelo tipado/policy de segurança.

## 8. Color Management e ajustes de imagem

`crates/cms/` inclui profiles, transforms, CLUT, LUT3D, gamut e pipeline; `crates/color/` blending e conversão; `crates/compose/src/adjust.rs` e `engine/src/adjust_*cmds.rs` implementam relações entre parâmetros/UI/render.

**Para Petunia:** estudar AST/params de Levels, Curves, Gradient Map, Hue/Saturation, Selective Color, Lookup, Blend If e masks. Color Management existente do Petunia tem direção LittleCMS; PhotoCraft CMS própria não deve ser segunda fonte da verdade de perfis/ICC. Comparar gamuts/proofs e compatibilidade de preview/export, e só então portar algoritmo isolado.

## 9. Automation, MCP e plugins

[`crates/automation/src/lib.rs`](https://github.com/storytold/photocraft/blob/5ecc860ac1abf8fa7e3f218c0474cdcce49fa6c7/crates/automation/src/lib.rs) indica `PhotocraftMcp` baseado no SDK `rmcp`, `Headless`, JSON-lines rpc, bridge remoto e `AuthorizedWorkspace`. [`docs/control-protocol.md`](https://github.com/storytold/photocraft/blob/5ecc860ac1abf8fa7e3f218c0474cdcce49fa6c7/docs/control-protocol.md) documenta **token de autenticação antes de executar comandos**, token file e read/write roots de automação. É referência especialmente boa para segurança de agentes: nunca expor control channel aberto sem autenticação/permissões.

[`crates/plugins/src/lib.rs`](https://github.com/storytold/photocraft/blob/5ecc860ac1abf8fa7e3f218c0474cdcce49fa6c7/crates/plugins/src/lib.rs) usa wasmi com sandbox, sem imports, fuel, memory limits, call deadlines e fresh instance por band de pixels. Payload `f32` em bandas bounded, na representação de color mode do documento. Útil para estimar custos de plugins raster e contratos de `Apply` atômico.

**Diferença vs VectorCraft:** runtime compartilha ideias, mas plugin vector trabalha objetos, enquanto PhotoCraft processa banda de pixels. No Petunia, criar Host API com capabilities **`DocumentObjects` versus `PixelTiles`** e validação específica; não forçar API única que exponha estruturas internas.

## 10. Principais lacunas e trade-offs

- PhotoCraft usa egui e state handlers de UI próprios: adaptar somente behavior, UI state machine e parâmetros via Qt/QML.
- Camadas, effects e clipping já têm Core Petunia: não gerar modelo de segundo documento.
- Compatibilidade PSD e precisão visual precisam testes sintéticos/headless, não confiança cega nas taxas relatadas.
- CPU+GPU paralelos aumentam esforço; um oracle CPU robusto tem vantagem mesmo quando shipping GPU não estiver pronto.
- Algumas APIs de brush pressupõem px e limits do PhotoCraft; no Petunia informar documento/pixel ratio, rotação, zoom e pressure normalization.
- Plugin sandbox sem FS/network serve a filtros puros; ações que importam/exportam files exigem Host API capabilities e autorização explícitas.
- `AGENTS.md` upstream fixa política de clean-room/never-crash; verificar localmente regras/licenças do código adaptado, não copiar apenas a linguagem do documento.

## 11. Plano de provas

**PC-01 COW tiles:** 256² sparse, unset tiles, default-mask pixels, clone cost, partial dirty/upload, Undo/Redo, memory budgets.

**PC-02 Brush parity:** dabs com pressure/flow/opacity/texture/scatter seeded e stroke interrupted; preview final = commit.

**PC-03 Compose oracle:** conjuntos sintéticos de 8/16/32-bit, alpha, blend modes, effects, mask, clipping, pass-through, linear versus display RGB; opções de compatibilidade nomeadas.

**PC-04 PSD I/O:** import → PTND → export, unknown blocks, untrusted lengths, layer bounds, unsupported features diagnostics, fuzzy corpus; não prometer roundtrip perfeito.

**PC-05 Native PTND save:** blobs deduplicados, atomic rename/recovery, migration, hash collision handling e input resource limits.

**PC-06 Agent automation:** token/permissions, folder read/write roots, undo atomic, job cancellation, UI inspection/keyboard, transitive plugin sandbox, no untrusted file exfil.

**Referências Petunia:** [Brush/Raster](#/docs/02-engine/brush-raster.md) · [Render](#/docs/03-render/compositing-effects.md) · [PTND](#/docs/00-architecture/ptnd-format.md) · [I/O Plugins](#/docs/02-engine/io-jobs-plugins.md).
