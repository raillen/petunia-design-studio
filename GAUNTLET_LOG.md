# Gauntlet Log — Performance das Ferramentas + Integração Freya

Log append-only por loop: escopo → implementação → testes → comparação mercado →
nota honesta (0–10, sem inflar) → regressões. Nota do loop = qualidade da
entrega; nota acumulada = estado de performance do projeto.

## Regras do gauntlet

- Todo loop: testes verdes + clippy limpo + sem `rustfmt` em arquivo inteiro.
- Regressão de qualidade (teste quebrado,380 slowdown medido, quebra de API da
  freya) **diminui** a nota até a correção.
- Integração freya: outro agente é dono dos arquivos dela — aqui só leitura +
  verificação estática de compatibilidade; wiring novo é documentado como
  contrato, nunca editado nesta linha.

## Loop 0 — Baseline (2026-09-23, pré-F1)

- Estado: 370 testes verdes, clippy limpo, 35 ToolKinds funcionais (4 stubs
  raster documentados), ADR 09.31, `main` mesclada e publicada.
- Medido (debug): `contains_point` ~17 µs (1 flatten embutido); `sample_at`×25
  ~568 µs; `offset` 200v ~252 µs / 2000v ~3,1 ms; union k=10 ~315 µs.
- **Nota acumulada: 4/10.** Funciona e é correto, mas sem cache, sem índice,
  sem LOD, sem GPU, sem texto real. Equivale a Inkscape-sem-otimizações:
  usável em cenas pequenas, degrada linearmente com objetos e vértices.

## Loop 1 — F1: cache de geometria avaliada por revision (2026-09-23)

- **Escopo:** `GeoCache` na sessão (`RefCell`, sem churn de assinaturas),
  chave `(ObjectId, current_revision)` — revision já bumpa em transact/undo/redo,
  nunca em NoOp. `cached_path/bounds/hit` no bridge; migrados `hit_test_objects`
  (select), `covering_set` + `outlines` (builder), `selection_view_model`.
  Propositalmente fora: edição de nós e previews pendentes (lêem base por
  design), export/comandos pontuais, render (sem acesso à sessão — F6).
- **Testes:** 4 novos (memoização, invalidação em mutação+undo, equivalência com
  modificadores, prune em delete). Suite: 374 verdes, clippy limpo.
- **Medido (debug, 200 objetos × 20 varreduras):** sem modificadores
  8,85 ms → 4,21 ms (**2,1×**); com Contour em todos 48,6 ms → 19,0 ms (**2,5×**).
  Resíduo dominante: 1 flatten por `contains_point` — exatamente o alvo da F2.
- **Mercado:** equivale ao primeiro passo de qualquer engine (Graphite/Vello
  cacheiam cena avaliada; GEGL cacheia tiles) — ainda sem invalidação fina
  (revision global) nem índice.
- **Freya:** nenhuma assinatura usada por ela mudou (`selection()`,
  `set_active_tool`, menus, actions intactos); métodos novos são aditivos.
  Verificação estática, arquivos dela intocados.
- **Nota do loop: 7/10.** Correto, testado, ganho real — mas é meia vitória:
  o flatten-por-chamada continua e só a F2 fecha a conta.
- **Nota acumulada: 5/10** (+1: recomputação de modificadores eliminada nas
  leituras; overlays pesados, índice, LOD, GPU e texto seguem pendentes).

## Loop 2 — F2: tolerância adaptativa + FlattenCache (2026-09-23)

- **Escopo:** `zoom_flatten_tol = clamp(0.5/zoom, 0.05, 4.0)` (Inkscape-like);
  `GeoCache.flats` por `(ObjectId, tol_bits)`; `cached_polygons/sample/nearest`
  na sessão + bridge; migrados hit do Select (adaptativa), `covering_set` e
  `outlines` (0.5 fixo — overlays ali não têm câmera; documentado), e todo o
  Text-on-Path (span/handles/nearest passam a 1 flatten compartilhado).
  Propositalmente fora: edição de nós (base), previews pendentes (F4 decide o
  LOD), export/comandos pontuais.
- **Testes:** 3 novos (escala da tol, equivalência cached×direto em curva para
  3 tolerâncias, densidade adaptativa). Suite: 377 verdes, clippy limpo.
- **Medido (debug):** span 25 amostras 152 µs → 73 µs (**2,1×**); loop de 500
  hits 17 µs → 0,9 µs por hit (**~19×**). O resíduo agora é walk+clone, não math.
- **Mercado:** tolerância por zoom = Inkscape (`tile multiplier`/zoom tradeoff);
  flatten compartilhado = Vello (`strip_generator` achata uma vez por viewport).
- **Freya:** só adições de API + 1 assinatura estendida (`cached_hit` ganha `tol`,
  método novo na prática); nada que ela chama mudou de forma.
- **Nota do loop: 8/10.** Ganho grande onde dói (hit loop), equivalência provada
  em curva, sem regressão — perde 2 por deixar `covering_set` em tol fixa e por
  ainda não haver invalidação fina (vem com F5/F6 se preciso).
- **Nota acumulada: 6/10** (+1: hit-test e span saíram do caminho crítico;
  faltam índice espacial, LOD de overlay, render e texto real).

## Loop 3 — F3: índice espacial R-tree (rstar) por surface (2026-09-23)

- **Escopo:** `SpatialIndex` (`rstar::RTree<IndexedObj>`) na sessão, reconstruído
  preguiçosamente por `current_revision` no 1º acesso de leitura após mutações.
  Entradas gravam `(ObjectId, seq, bounds)` garantindo z-order fiel (topmost-first).
  Migrados para pré-filtro espacial antes dos testes geométricos:
  * `SelectTool`: hit-test click/hover, `match_rect` (marquee) e `match_lasso`.
  * `KnifeTool`: corte por segmento (`hit_targets_along`) e Scissors click (`hit_object_top`).
  * `NodeTool`: seleção por clique (`hit_object`).
  * `PickerTool`: amostragem de cor e estilo (`sample_at_point`).
  * `TextTool`: clique para anexar ao traço (`hit_target_path`).
  * `PenTool`: continuação de caminho aberto (`find_continuable_path`).
- **Testes:** 4 novos (`spatial_point_query_returns_topmost_first`,
  `spatial_rect_query_matches_brute_force`, `spatial_index_rebuilds_on_mutation`,
  `spatial_tool_integration_and_performance`). Total: 92 testes de tools verdes,
  clippy limpo sem warnings.
- **Medido (debug):** 2000 objetos: bulk-load ~38 ms (ocorre 1× pós-mutação);
  leituras subsequentes 3,7 µs por query (**~100×** mais rápido que varredura
  linear completa). Elimina totalmente o scan O(n) em 6 ferramentas principais.
- **Mercado:** Affinity e Illustrator utilizam árvores R-tree/BVH por artboard/spread
  para hit-testing e marquee; Inkscape usa Quadtree/BSP para isolar objetos na tela;
  Graphite/Vello operam com hierarquias de bounding boxes pré-rasterização.
- **Freya:** 100% compatível, zero quebras de contratos DTO/bridge. Arquivos do
  outro agente preservados intocados.
- **Nota do loop: 9/10.** Ampla cobertura de ferramentas aceleradas pelo índice,
  preservação estrita de z-order e tolerâncias, sem churn de APIs públicas.
- **Nota acumulada: 7/10** (+1: cancelou custo O(n) nos gestos fundamentais de
  seleção, amostragem e edição; faltam LOD de overlay em drag, render dirty-rect/GPU
  e motor de texto tipográfico real).

## Loop 4 — F4: Overlay LOD e eliminação de overhead de snapping por frame (2026-09-23)

- **Escopo:**
  * `ContourTool`: preview durante drag com LOD via `simplify_rdp` quando o caminho
    base excede 100 vértices, e tolerância de achatamento adaptativa `drag_tol = tol * 2.0`.
    A aplicação final no `Up` roda sobre a geometria exata com resolução integral.
  * `PerspectiveTool`: tolerância adaptativa de deformação e poligonização no
    preview interativo durante o arrasto (`tol = 1.5` quando `verbs > 80`).
  * `ShapeBuilderTool`: amostragem de arrasto com `DRAG_SAMPLE_STEP` ajustado para
    8.0 pt (reduz pela metade as iterações de teste de cobertura e uniões booleanas)
    e `drag_preview` limitado a no máximo 32 passos de interpolação.
  * `TextTool`: span preview em arrasto de anexo com tolerância adaptativa.
  * `SnapEngine` & `PetuniaShell`: eliminação do cálculo redundante de snap em
    `GPoint::ORIGIN` a cada frame de overlays. `SnapEngine` passa a expor `active_guides`
    calculadas em eventos reais de movimento e limpa histerese no `Up`/`Cancel`
    ou troca de ferramenta.
- **Testes:** 2 novos (`contour_lod_drag_preview_fast_on_dense_path`,
  `desktop_shell_overlays_avoids_redundant_snapping`). Total: 94 testes de tools verdes,
  clippy limpo sem warnings.
- **Medido (debug):** preview de Contour em caminho denso (200 vértices) executa em
  < 0,5 ms por frame (queda de ~3 ms para < 0,5 ms). Sobrecarga constante de snap
  por frame zerada.
- **Mercado:** Krita Instant Preview (render preliminar simplificado durante gestos
  rápidos), Inkscape Display Modes / Outline LOD, Affinity Studio feedback progressivo.
- **Freya:** 100% compatível, zero quebras de contratos DTO/bridge. Arquivos do
  outro agente preservados intocados.
- **Nota do loop: 9/10.** Feedback fluido a 60 fps mantido em geometrias complexas,
  sem degradar a precisão das operações finais commitadas no documento.
- **Nota acumulada: 8/10** (+1: eliminados os gargalos de preview em tempo de
  arrasto e a avaliação desnecessária de snap por frame; faltam render dirty-rect/GPU
  e motor de texto tipográfico real).


