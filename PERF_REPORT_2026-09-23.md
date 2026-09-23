# Relatório de Performance das Ferramentas — Achados, Referências e Plano Concreto

- **Data:** 2026-09-23 · **Branch:** `refactor/tools-funcionamento-2026-09-22`
- **Escopo:** 35 ToolKinds + EffectChain (4 ModifierKinds) + texto vetorial + render.
- **Método:** auditoria estática linha-a-linha + micro-benchmarks em debug
  (valores absolutos caem ~5–20× em release; o **escalonamento** é o que importa).

## TL;DR

Três gargalos dominam tudo, todos com a mesma raiz — **recomputar geometria
do zero a cada chamada, sem cache e sem índice espacial**:

1. **Sem memoização de geometria avaliada.** Cada `hit_test`, `overlays`,
   `selection` e `export` reflattena paths e re-roda modificadores
   (offset/warp/booleanos). Medido: `contains_point` ≈ 17 µs/chamada em debug
   **incluindo 1 flatten**; hover sobre 500 objetos ≈ 8,5 ms por evento `Move`.
2. **Overlays pesados por frame.** Preview de Contour roda `offset_path`
   inteiro por `Move` (252 µs/200 vértices, 3,1 ms/2000 em debug); Perspective
   re-warpa; ShapeBuilder refaz booleanos (drag amostra até 257 pontos);
   Text-on-Path reflattena o alvo **25× por frame** (568 µs em debug).
3. **Sem índice espacial, sem LOD, sem tiling.** Hit/marquee são O(n) com
   flatten exato; render é `fill_rect` por objeto em CPU sem culling/dirty-rect;
   texto não tem shaping nem cache de fonte (métrica estimada `0.55×size`).

Nada disso exige GPU para a fase 1. Ordem de ataque: cache + índice + LOD
(ganho 10–100× nos gestos), depois render/GPU e texto real.

## 1. Evidência medida (debug, `petunia_design_geometry`, indicativo)

| Operação | n=20 | n=200 | n=2000 | Leitura |
|---|---|---|---|---|
| `to_polygons(0.5)` | 24 µs | 19 µs | 184 µs | Linear, barato isolado |
| `offset_path` expand | 66 µs | 252 µs | 3,1 ms | Superlinear (kurbo stroke) — **não repetir por frame** |
| `contains_point` ×1000 | — | 17 ms total | — | Cada chamada reflattena: ~17 µs |
| `sample_at` ×25 (modelo do span de texto) | — | 568 µs | — | 25 flattens redundantes |
| `union` incremental k=10 rects | — | 315 µs | — | OK pontual, ruim em loop de drag |
| `warp_path_to_quad` | — | 80 µs | — | OK pontual, ruim por frame |
| `rect_corners` ×1000 | — | 0,46 ms total | — | Irrelevante (0,5 µs) |

Conclusão dos números: **nenhuma primitiva isolada é lenta; a repetição sem
cache é o custo.** 500 objetos × 17 µs = 8,5 ms por `Move` só em hit-test
(debug; ~1–2 ms em release — ainda metade do orçamento de 60 fps).

## 2. Achados por severidade (com local exato)

### P0 — Geometria avaliada sem cache (raiz de quase tudo)

- `document_object.rs:211` — `evaluated_path()` clona (`to_path`) + re-avalia
  a cadeia **sempre**; cadeia vazia custa 2 clones cheios.
- `document_object.rs:217` (`evaluated_bounds`), `:242` (`hit_test`),
  `:231` (`sampled_opacity` + `modifiers.rs:183` sort+`Vec` por chamada).
- Leitores em loop: `select.rs:321,376` (hover/click O(n) com flatten),
  `session.rs:663` (`selection_view_model` → `evaluated_bounds` por selecionado,
  chamado por `select/gradient/perspective/measure/properties` overlays),
  `shape_builder.rs:260,274,424`, export pontual (ok).
- Não existe `revision`/dirty-bit por objeto; o `Evaluator` só cacheia
  `DocSummary` (`evaluation/lib.rs:80`, fingerprint O(doc) — pior que o problema).

### P0 — Overlays recomputam pesado por `Move`/frame

- `contour.rs:284` — `pending_outline` roda `offset_path` + `to_polygons` por frame.
- `perspective.rs:239` — `warp_path_to_quad` + flatten por frame.
- `shape_builder.rs:224,311` — `drag_preview` com até 64 amostras ×
  (`covering_set` + `region_polygons` + `union`) por `Move`.
- `text.rs:598` — `span_points` = 25× `sample_at`, cada um reflattena o alvo.
- `gradient.rs:335,369` — `effective_appearance()` clone + `String` por stop por frame.
- `desktop_shell.rs:115` — `snap_point(ORIGIN)` por frame mesmo parado.

### P1 — Sem índice espacial

- `covering_set` (`shape_builder.rs:250`) sem bbox pre-check (só `select.rs:734`
  e `node.rs:1227` têm); grep por `quadtree|BVH|rstar` = zero.
- Marquee/lasso são O(n) baratos **porque** usam só `bounds` (correto manter),
  mas click/hover pagam flatten exato por objeto.

### P1 — Booleanos com pior caso quadrático em gestos

- `drag_regions` (`shape_builder.rs:167`): até 257 amostras × cobertura+região;
  `union` incremental (`:188,203`) e `intersect_all` pairwise
  (`selection_mask.rs:254`) são O(R²) chamadas ao overlay; cada chamada
  reconverte `GPoint⇄[f64;2]` (`boolean.rs:151`).
- `difference_many` já é 1 chamada (correto — batch 12).

### P1 — Tolerância de flatten fixa, cega a zoom

- `0.25`/`0.5` hardcoded em `modifiers.rs:140,147`, `offset.rs:78,176`,
  `shape_builder.rs:271`, `text.rs:26`. Com zoom-out, polígonos densos demais;
  com zoom-in extremo, erro visível. Todo mercado escala tolerância por zoom.

### P2 — Render CPU sem tiling/culling/LOD

- `scene.rs:20` ("no cached pixels"); `pixel_compositor.rs:290` lookup de
  máscara O(n) **por objeto** (O(n²) total); `fill_rect` O(área) em CPU;
  sem dirty-rect, sem culling além de bounds, vetor vira retângulo.
- Sem dependência GPU (`vello`/`wgpu` ausentes do workspace).

### P2 — Texto sem shaping, sem fonte, sem cache

- `text/layout.rs:46` — largura = `0.55 × size` estimada; sem HarfBuzz/rustybuzz,
  sem fontdb/font-loading, sem `SwashCache`; `story.rs:173` insert/delete O(n).
- `petunia_design_text` sequer tem crates de fonte no `Cargo.toml`.
- Text-on-Path herda tudo isso + 25 flattens/frame (item P0).

### P3 — Micro-alocações (fazer por último)

- `AppearanceStack` clonada por chamada (`effective_appearance`),
  `order: Vec + sort` por `sampled_opacity`, `lasso/pen_preview.clone()` por frame.

## 3. O que o mercado faz (referências verificadas)

| Software | Técnica que importa aqui | Fonte |
|---|---|---|
| **Krita** | **Instant Preview / LOD strokes**: feedback em canvas reduzido enquanto calcula o traço real em background; brush multithread + tiles | docs.krita.org (Instant Preview) |
| **GIMP/GEGL** | Grafo demand-driven + **tiles com copy-on-write, swap e compressão**, `parallel_distribute_area` multicore, cache 50% RAM, preview de operação antes de aplicar | developer.gimp.org, GEGL NEWS |
| **Inkscape** | **Display modes** (No Filters / Outline — até 10× mais responsivo), **tile multiplier**, threads de render, qualidade de blur por display, Simplify (Ctrl+L), clones/symbols | inkscape.org, fórum |
| **Graphite** (Rust, open) | **Node-graph não-destrutivo** (nossa EffectChain é o primo pobre correto) + **Vello GPU**; editor separa tooling de engine por mensagens | graphite.art, GitHub |
| **Vello** (Rust) | **Sparse strips + tiling**, CPU/GPU/híbrido, SIMD+multithread, `glyph_run` com cache de glifos | linebender/vello |
| **cosmic-text** (Rust) | `FontSystem` 1/app, `SwashCache` de glifos, `RunCache`, `Shaping::Basic` (barato) vs `Advanced` (fallback/complexo) | docs.rs, GitHub pop-os |
| **Photoshop/Affinity** (prática consagrada) | Tiles + compositing GPU + display progressivo; Affinity: Metal/compute incremental |(doutrina da indústria) |
| **rstar/spart** (Rust) | R-tree/quadtree prontos: point lookup ~177 ns/100k itens, bulk-load 2000 em ~230 µs | docs.rs benchmarks |

Padrão comum: **ninguém recalcula tudo por frame** — cache + invalidação +
degeneração graciosa (LOD/outline) + índice espacial + tiles/GPU por último.

## 4. Plano concreto (fases, arquivos, aceite)

### F1 — Cache de geometria avaliada por objeto (maior ROI, 1 batch)

- Onde: `document_object.rs` + `mutator.rs` + `session.rs`.
- O quê: `EvaluatedCache { revision: u64, path: GPath, bounds: Option<[f64;4]> }`
  por objeto; `DocumentMutator` bumpa `session.current_revision` (já existe!)
  a cada `transact`; `evaluated_path()` só recomputa se
  `cache.revision != session revision` — mas `DocumentObject` não vê a sessão.
  Implementação limpa: cache vive na **sessão**
  (`HashMap<ObjectId, (u64, GPath, bounds)>`), `bridge.evaluated_path(id)` consulta;
  ferramentas migram de `obj.evaluated_path()` para o bridge (assinaturas já
  recebem `bridge`). Invalidação = revision global (simples) ou por-objeto via
  `ChangeSet` (refinado depois).
- Aceite: hover sobre 500 objetos < 2 ms; teste com 2000 vértices × Contour.
- Custo: 1 batch, sem mudança de formato.

### F2 — Tolerância adaptativa a zoom + FlattenCache (1 batch, junto da F1)

- Onde: `geometry/` novo `flatten_cache.rs` + call sites com `0.25/0.5` fixos.
- O quê: `tol = clamp(0.5 / zoom, 0.05, 4.0)` (regra Inkscape-like); cache por
  `(PathHash, tol_bucket)` com hash dos verbs (FxHash, não JSON).
- Aceite: zoom-out 10% não multiplica vértices; teste pinna tolerância por zoom.

### F3 — Índice espacial rstar por surface (1 batch)

- Onde: novo `spatial_index.rs` em `application` ou `shell/canvas`; crate `rstar`.
- O quê: R-tree de `(bounds, ObjectId)` reconstruído por revision;
  `hit_test_objects`, `covering_set`, marquee usam query por ponto/rect antes
  do teste exato. Benchmark docs.rs: lookup ~177 ns/100k.
- Aceite: hit em 10k objetos < 0,5 ms; teste de regressão com cena sintética.

### F4 — Overlay LOD: preview barato no drag, full no Up (1 batch, padrão Krita/Inkscape)

- Onde: `tools/*::overlays` + `desktop_shell.rs:115`.
- O quê: durante drag ativo, previews custosos (offset/warp/boolean/span)
  degeneram para **bbox + polyline grosseira** (tol ×8); cálculo full só em
  hover parado e no `Up`. `snap_point(ORIGIN)` só sob gesto ou mudança de snap.
- Aceite: `Move` com Contour em path 2000v < 8 ms (era ~3 ms só offset em debug).

### F5 — Boolean incremental + memo de outlines no Builder (1 batch)

- Onde: `shape_builder.rs`, `selection_mask.rs`.
- O quê: `outlines` por `(ObjectId, revision)` cacheados na sessão (usa F1);
  drag amostra a cada 8 px em vez de 4; `union` acumula em 1 chamada final
  (como `difference_many`); bbox early-out antes de `region_polygons`.
- Aceite: drag longo em 10 formas < 16 ms/frame.

### F6 — Render: dirty-rect + culling + lookup O(1) (1 batch); Vello depois (epic)

- Onde: `render/`.
- O quê: mapa `clip_mask_id → objeto` (mata o O(n²)); culling por viewport;
  dirty-rect por `ChangeSet` (só re-renderiza o que o changeset tocou);
  **epic separado:** backend `vello_cpu` (SIMD+threads, sem GPU exigida) e
  `vello_hybrid` depois — Graphite prova o caminho em Rust.
- Aceite: pan/zoom sem re-render total; teste de dirty-rect.

### F7 — Texto real: cosmic-text (epic, 2–3 batches)

- Onde: `petunia_design_text` + `render` + `tools/text.rs`.
- O quê: `fontdb` (descoberta) + `rustybuzz/harfrust` via **cosmic-text**
  (`FontSystem` 1/app, `Buffer` por objeto com cache por revision,
  `SwashCache` de glifos); `Shaping::Basic` default, `Advanced` sob demanda;
  text-on-path posiciona `layout_runs()` sobre `sample_at` (troca os 25 flattens
  por 1 layout cacheado); hit-testing de texto via runs (click detection do cosmic).
- Aceite: texto multilíngue correto (UDHR test como referência), layout O(linhas
  visíveis), glifos sem re-rasterizar por frame.
- Não-fazer: shaping próprio, font-loading próprio, `0.55` para sempre.

### F8 — Micro-opts (contínuo, nunca antes da F1–F5)

- `SmallVec`/slices em vez de `Vec` temporários, `&str`/token interning em vez
  de `String` por stop, `sort_unstable`, `order` reutilizável. Ganho ~5–15%.

## 5. Orçamento de aceite (60 fps = 16,6 ms/frame)

| Gesto | Orçamento | Hoje (debug, estimado) |
|---|---|---|
| Hover hit-test, 500 objetos | < 2 ms | ~8,5 ms |
| Drag overlay Contour, 2000v | < 8 ms | ~3 ms só offset + overhead |
| Drag Builder, 10 formas | < 16 ms | quadrático, estoura |
| Re-render pan | só dirty-rect | total |

## 6. Riscos e não-fazer

- **Não paralelizar (rayon) antes do cache:** paralelismo em cima de recomputação
  é enxugar gelo; GEGL paraleliza tiles, não flattens redundantes.
- **Não GPU antes de F1–F6:** Vello sem cache de cena repete o mesmo desperdício no GPU.
- **Não `unsafe`/SIMD manual:** `fearless_simd` via vello cobre quando chegar lá.
- **Invalidate certo:** cache sem invalidação correta vira bug fantasma — todo
  cache proposto ancora em `current_revision`/`ChangeSet`, nunca em tempo.
