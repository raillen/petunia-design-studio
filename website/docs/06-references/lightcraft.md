# LightCraft — auditoria técnica para Photo persona, color, preview e máscara

**Snapshot:** [storytold/lightcraft@832b8ada](https://github.com/storytold/lightcraft/tree/832b8ada51af1e18ddc2eba87fbfe0485d150fc7), 2026-10-08. Inspeção estática de manifests, diretórios e fontes centrais. **Não foi executado, benchmarkado nem validado com câmeras/fotos reais.** Indicadores de paridade e timings do README/documentação são declarações do upstream.

**É permitido copiar/portar diretamente algoritmos e infraestrutura compatíveis**, inclusive preview cache e pipeline, com proveniência. `crates/segment` tem licença Apache-2.0-only e os pesos SAM 3 seguem licença distinta — não incluí-los inadvertidamente. Veja [Reutilização Direta](#/docs/06-references/code-reuse-policy.md).

## 1. Arquitetura e foco

Rust workspace edition 2024, v0.4.0, 20 crates, apps desktop/CLI/web, egui/eframe 0.36. Domínio é biblioteca fotográfica + revelação RAW **não destrutiva**, e não editor vetorial multicamadas. Crates:
- `raw`, `tiff`, `codecs`, `meta`, `color`, `raster`, `geom`: decode/metadata/geometry;
- `develop`, `pipeline`, `gpu`, `segment`: schema de ajustes, render CPU/GPU e IA;
- `catalog`, `preview`, `merge`, `scenes`, `fetch`, `sysmem`: biblioteca, caches, fotografia;
- `engine`, `mcp`, `ui-egui`: comandos, automação e interface.

**Uso no Petunia:** inspiração especialmente forte para uma futura **Photo persona / Develop workspace** e recursos de correção fotográfica integrados a layers raster. A biblioteca de catálogo inteira é opcional para um estúdio de design; não assumir que precisamos construir substituto de Lightroom.

## 2. Modelo de edição não destrutiva

[`crates/develop/src/settings.rs`](https://github.com/storytold/lightcraft/blob/832b8ada51af1e18ddc2eba87fbfe0485d150fc7/crates/develop/src/settings.rs): `DevelopSettings` versionado (`SCHEMA_VERSION=1`, serde defaults) com profile, white balance, light, tone curve, color adjustments, mixer, B&W, grading, effects, vignette, grain, detail, optics, geometry, crop, masks, red-eye, lens blur, enhancement e camera calibration.

**Boas ideias:** parâmetros semânticos e independentes da resolução, versões de schema, defaults neutros, grupos enable/disable e presets reutilizáveis. Em Petunia mapear como `PhotoDevelopEffect`/Adjustment stack com params tipados e migration, **sem transformar source de bitmap na saída corrigida**. Manter originais/recursos separados do cache renderizado e da preview.

**Cuidado:** defaults `serde(default)` e campos desconhecidos ignorados ajudam forward compatibility mas também podem descartar dados novos ao fazer roundtrip em versões antigas. PTND precisa política explícita `TooNew`, opaque preservation ou read-only, não herdar ignore-unknown como regra universal.

## 3. CPU pipeline e separação de estágios

[`crates/pipeline/src/lib.rs`](https://github.com/storytold/lightcraft/blob/832b8ada51af1e18ddc2eba87fbfe0485d150fc7/crates/pipeline/src/lib.rs) especifica input scene-referred linear Rec.2020, output display encoded sob `RenderRequest`, e estágios:

1. **Geometry**: orientation, lens distortion/CA/vignetting, perspective, crop, straighten, flip; resampling coordenado.
2. **Scene-linear**: white balance, exposure, dehaze, local tone, texture/clarity, masks/local adjustments.
3. **Tone map**: highlights/shadows, curve contrast e highlight desaturation.
4. **Colour**: vibrance/saturation/mixer/grading/B&W.
5. **Display**: gamut mapping/transfer, final curves/vignette/grain conforme pipeline.

O código declara `StageCache` para reutilizar etapas cujos parâmetros e inputs não mudaram (arrastar slider de exposição/cor não precisa reprocessar geometry). Spatial params dependem do **tamanho relativo à maior aresta** para manter similaridade preview/full-res export.

**Adaptação:** manter esses estágios sob um **Effect Graph evaluable por ROI**, convertendo corretamente para color-space da composição Petunia. Não guardar pixels desenvolvidos como únicos pixels da layer nem aplicar display transform antes da composição linear por conveniência. Diferenciar `PreviewQuality`, `ExportQuality` e `MaxError` sem alterar a semântica de ajustes.

**Testes:** preview 400px vs export full resolution, extreme aspect ratios, wide gamut, highlight rolloff, profile combinations, resampling/CA/optics, geometric change invalidates only necessary caches.

## 4. Mask pipeline: composição matemática simples e potente

[`crates/pipeline/src/masks.rs`](https://github.com/storytold/lightcraft/blob/832b8ada51af1e18ddc2eba87fbfe0485d150fc7/crates/pipeline/src/masks.rs) define cada Mask avaliada como alpha plane em output resolution, com geometria em coordinates normalizadas de foto orientada.

O cabeçalho mostra regras de composição:
- **Add:** máximo das alphas (max), diferente de alpha-over comum;
- **Subtract:** `a * (1-c)`;
- **Intersect:** `a * c`;
- inversão e amount aplicados por componente/mask.

**Importância:** User-facing Add/Subtract/Intersect precisa anunciar operador exato. Petunia usa masks em composições complexas; para cada contexto (selection mask, PixelLayer mask, Vector clip, Develop mask), definir operador próprio sem supor que max(Add) seja equivalente à união probabilística ou alpha-over.

**Aplicações:** brush mask, linear/radial gradients, color/luminance range, subject/sky via IA opcional, opções de feather/edge, combinação de máscaras e adjustment local. A estrutura normalizada é boa para operações de foto, mas NÃO deve virar a coordenada canônica de todos os masks de layout/canvas com dimensões absolutas em mm.

## 5. Preview/cache/job model

[`crates/preview/src/lib.rs`](https://github.com/storytold/lightcraft/blob/832b8ada51af1e18ddc2eba87fbfe0485d150fc7/crates/preview/src/lib.rs):
- Hash de conteúdo para detecção de duplicatas/chaves;
- LRU com orçamento de bytes para memória e disco;
- thumbnails cacheados por `(content hash, settings hash, size, renderer version)`;
- `JobPool` com prioridade para imagens visíveis e deduplicação por slot;
- camada de preview independente da GUI e compartilhável por CLI/MCP.

**Altíssima aplicabilidade:** thumbnail de Document pages/artboards, previews de assets, Image References, RenderedEffects, zoom proxies, file browser e export preview. Mantém ROI/state hash e generation token para invalidar resultados obsoletos.

**Modelo proposto:** `PreviewKey {resource_hash, revision, effect_hash, dimensions, color_space, renderer_version}`, `JobPriority`, `CancelToken`, `Generation`, LRU memory/disk; não usar apenas path+mtime como identidade confiável nem bloquear UI aguardando thumbnail fora de vista.

## 6. Catálogo resiliente: snapshot + append-only journal

[`crates/catalog/src/lib.rs`](https://github.com/storytold/lightcraft/blob/832b8ada51af1e18ddc2eba87fbfe0485d150fc7/crates/catalog/src/lib.rs) explica `Catalog::apply(Op) → inverse Op`, replay determinístico, Undo/Redo e persistência.

[`crates/catalog/src/journal.rs`](https://github.com/storytold/lightcraft/blob/832b8ada51af1e18ddc2eba87fbfe0485d150fc7/crates/catalog/src/journal.rs) descreve:
- `catalog.snap` JSON versionado com seq de snapshot, trocado atomically (temp/fsync/rename);
- `catalog.log` append-only JSON lines, seq + CRC-32;
- recuperação de última linha rasgada, append parcial, snapshot/log sobreposto, arquivo danificado isolado;
- background snapshot/compaction em store compatível.

**Vale estudar para:** autosave/recovery do PTND, histórico de Commands e crash resilience, mas **não trocar o formato PTND** por `catalog.log`. O estado Petunia tem árvore, raster tiles, conteúdo externo, assets, versão, cenas; projetar journal tipado e atomicidade inter-resource própria. CRC detecta corrupção acidental, não autentica adversários nem substitui hashes criptográficos para blobs.

**Importante:** o catálogo é biblioteca de fotos; import de regras de rating/flag/album só faz sentido se houver persona Asset Library/DAM. Não levar complexidade de Library para a inicialização/editor de canvas.

## 7. GPU compute, CPU oracle e fallback

[`crates/gpu/src/lib.rs`](https://github.com/storytold/lightcraft/blob/832b8ada51af1e18ddc2eba87fbfe0485d150fc7/crates/gpu/src/lib.rs) usa o `lightcraft-pipeline` como referência; WGSL compute ports leem shared resolved parameters. Stages ainda sem kernel podem cair para CPU no mesmo render. GPU falha/está desligada ou device limits são excedidos → `None` e fallback CPU, com motivo rastreável.

[`docs/gpu-pipeline.md`](https://github.com/storytold/lightcraft/blob/832b8ada51af1e18ddc2eba87fbfe0485d150fc7/docs/gpu-pipeline.md) documenta seleção seletiva de backend para evitar crash no driver na enumeração de GPU, coverage mask calculada em CPU f64 na borda de transform projective e equivalência tolerada com resultados GPU.

**Lição para Petunia:** não selecionar a GPU indiscriminadamente; não confundir cálculo de render com apresentação Qt. Planejar deterministic CPU fallback, device-loss recovery, ROI budgets, telemetry de fallback e teste de qualidade das bordas em perspective/warp. Mac/Linux/Windows necessitam política de backend específica que deve respeitar nossa render stack, não copiar as defaults sem teste.

**Limite verificado:** comentário do `gpu/src/lib.rs` informa que no target wasm32 o caminho GPU ainda não está disponível (fallback CPU), apesar de LightCraft ter frontend web. Não inferir WebGPU compute pronto para todos os targets.

## 8. RAW, metadata, color calibration, export

[`crates/raw/src/lib.rs`](https://github.com/storytold/lightcraft/blob/832b8ada51af1e18ddc2eba87fbfe0485d150fc7/crates/raw/src/lib.rs) implementa reconhecimento/decodificação de DNG, Canon CR2/CR3, Nikon NEF, Sony ARW, Fuji RAF/X-Trans, Panasonic RW2, Pentax PEF, Olympus ORF e outros variants, incluindo demosaic, preview embutido e camera colour matrices. Comentário reconhece formatos/variantes ainda não suportados/verificados. Módulos de `meta` tratam EXIF/IPTC/XMP/tags; `merge` trabalha panorama/HDR; `pipeline` filtros.

**Decisão de produto sugerida:** RAW import em Petunia como `ImageResource`/source imutável + `DevelopEffect` por adapter, em vez de reimplementar já todo um revelador de biblioteca fotográfica. Avaliar `lightcraft-raw` como dependência ou fonte de algoritmo isolado após corpus de câmeras, riscos de segurança, qualidade perceptual, resolução e tamanho do binário.

**Priorizar:** `embedded_preview`/thumbnails e import seguro, white balance, lens correction e tone curve. HDR merge/panorama avançado só quando persona fotográfica justificar.

## 9. SAM 3 e segmentação assistida

[`crates/segment/src/lib.rs`](https://github.com/storytold/lightcraft/blob/832b8ada51af1e18ddc2eba87fbfe0485d150fc7/crates/segment/src/lib.rs) implementa SAM 3 em Rust/Candle. Segundo source: encoding da foto compartilhado para diferentes prompts; click positivo/negativo e texto; GPU Metal em macOS e CPU elsewhere naquele commit; weights **não incluídos**.

**Licenças:** `crates/segment` é **Apache-2.0 only** por port de Hugging Face Transformers. Os pesos `facebook/sam3` ficam sujeitos à **SAM License**, uma licença separada não-OSI e download iniciado pelo usuário. Tratar software e pesos separadamente: checar termos de redistribuição, finalidade comercial, dados pessoais e opt-in. Não embutir pesos nem prometer execução local em hardware modesto sem profiling.

**Integração responsável no Petunia:**
`Subject Select / Background / Sky` opcional como Job do Engine, com `Capabilities::AiSegmentation`, status de disponibilidade, download claro e resultado como `MaskProposal` (nunca Commit até usuário confirmar). Alternativas manuais Marquee/Lasso/Brush/Color Range funcionam sempre sem IA.

## 10. MCP e command-first

[`crates/engine/src/lib.rs`](https://github.com/storytold/lightcraft/blob/832b8ada51af1e18ddc2eba87fbfe0485d150fc7/crates/engine/src/lib.rs) centraliza comandos `photo.rate`, `develop.set`, `album.create`, `mask.add`; `RenderJob` é `Send`. [`crates/mcp/src/lib.rs`](https://github.com/storytold/lightcraft/blob/832b8ada51af1e18ddc2eba87fbfe0485d150fc7/crates/mcp/src/lib.rs) disponibiliza `Remote` + `Headless` via protocolo MCP stdio, com controle desktop em loopback JSONL.

**Petunia:** normalizar comandos de GUI, CLI, actions, plugin Host e MCP num dispatcher de Engine com tipos DTO, schemas, permission scopes e uma Transaction semântica. `ui.widgets` ou inspect de estado pode expor controle automatizado acessível sem vazar pixels/documentos privados sem autorização.

## 11. Restrições e problemas a resolver antes de integrar

1. LightCraft é photo developer/library, e não possui o SceneGraph/Layer compositing completo que Petunia precisa para design.
2. Pipeline scene linear Rec.2020 pode demandar conversões para color space/document profile Petunia — aplicar CMM corretamente.
3. Browser build roda GPU fallback CPU no estágio observado.
4. Compatibilidade de RAW depende do fabricante/modelo; não anunciar suporte por extensão como prova completa.
5. ML optional com weights sob licença não-OSI requer decisão jurídica e packaging separados.
6. Catálogo append-only é subsistema de biblioteca e não substitui PTND/History.
7. Definições de máscara em coords relativas não substituem unit system de vetores/layout.
8. Claims de desempenho/paridade são upstream e precisam bench no hardware e arte representativos do Petunia.

## 12. Experimentos propostos

**LC-01** esquema de ajuste não destrutivo e cross-resolution results (proxy vs export).
**LC-02** StageCache/PreviewCache integrado a RenderSnapshot, cancel/generation e low-memory.
**LC-03** CPU/GPU parity com color profiles, warp, masks e device loss.
**LC-04** máscara composta Add/Subtract/Intersect com operator policies contextualizadas (photo vs layer mask).
**LC-05** import RAW seguro, metadata, unsupported model diagnostics, large files e raster decoded budget.
**LC-06** crash recovery e journal por Transaction sem substituir PTND.
**LC-07** AI masks opt-in e licença/resources auditable, apenas se for milestone aprovado.

**Ligações Petunia:** [Color Management](#/docs/02-engine/color-management.md) · [Paint Evaluation](#/docs/03-render/paint-evaluation.md) · [Persistência](#/docs/02-engine/persistence-recovery.md) · [Smart Color](#/docs/04-ui/smart-color-interaction.md).
