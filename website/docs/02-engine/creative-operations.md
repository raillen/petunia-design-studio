# Creative Operations — contratos do Engine

**Status:** especificação de Engine para as ferramentas aceitas. A UI não implementa algoritmos; todos oferecem avaliação headless, operações tipadas, cancelamento quando caro, preview/commit consistentes e materialização explícita.

## Interfaces

~~~text
Core authoring snapshot
  ↓
Geometry / Spatial / Color / Layout / Render evaluation
  ↓
Typed Derived Result
  ├── preview: no mutation
  └── command: prepared transaction → atomic commit
~~~

Resultados têm versão semântica quando uma mudança de algoritmo poderia mudar a obra já salva.

## Smart Path

- **Direct Bend:** escolher t* no segmento por nearest-point search; resolver deslocamento preservando endpoints e continuidade solicitada com curva Bézier cúbica. Usar solução limitada/least-squares quando necessário, validar singularidades e max deviation. Não usar simples handle guess que salte no commit.
- **Smart Delete:** remover node e reconstruir segmento substituto por curve fitting com erro máximo definido no espaço documental, preservar tangentes quando exigidas. Se não houver solução na tolerância, oferecer proposta com erro e operação Hard Delete explícita; nunca deformar silenciosamente.
- **Clean Vector:** pipeline de análise (near-duplicates, zero-length, collinear, tiny loops, gaps, self-intersections) → relatório com severidade → propostas idempotentes → preview → 1 Transaction. *Análise* não altera geometria.
- **Select Similar:** query scene index/properties, comparar cor em espaço/perceptual apropriado, tolerâncias de estilo, fontes e tipos, respeitar escopo e hidden/locked. Seleção = Session State.

## Smart Region

1. normalizar transformed paths para espaço comum;
2. flatten adaptativamente sob tolerância apropriada;
3. encontrar interseções e subdividir edges;
4. construir **planar arrangement** (grafo de vértices/arestas/faces no plano);
5. identificar faces conforme EvenOdd/NonZero e provenance;
6. gerar hits e máscaras de preview;
7. mapear ações Build/Paint/Weave para operações semânticas;
8. materializar paths somente por Expand.

**Gap detection:** procurar pares endpoints ou segmentos próximos com regras de geometria/ângulo/escopo e distância máxima. Virtual Bridge muda apenas fechamento da região avaliada; Close Geometry é Command separado. Impedir fechamento através de outros segmentos sem diagnóstico. Ambiguidade exige escolha, não decisão automática.

**Intertwine:** não é apenas Paint invertido. Usa split/occlusion region por crossing com precedence local; precisa reconstruir recortes/masks durante avaliação. No caso de N objetos, detectar ciclos de precedência regionais inconsistentes; não alterar z-order global.

## Smart Distribution

**Repeat:** gerar transforms por fórmula canônica e ordem estável. Mirror com reflection/axis, radial com centro e ângulo, grid com row/column + offset. Não duplicar source no SceneGraph.

**AlongPath:** parametrização por comprimento de arco; amostrar t para distância desejada, calcular tangente/normal e orientação; cantos/cusps têm policy explícita. Distância constante segue arco, não parâmetro t uniforme.

**Blend:** interpolar transforms/decompositions e paints em espaço escolhido, steps/easing/spacing/direction, fonte com tipos compatíveis. Geometry morph requer contour correspondence; quando falta, fallback visual deve ser declarado **Crossfade** e não vendido como morph geometricamente verdadeiro. Não armazenar intermediários como fonte.

**Scatter:** PRNG seeded estável, densidade, spacing, collision policy, rotation/jitter e bound de contagem; cancelar/limitar alocações.

## Smart Color

Palette extraction: amostragem/profile conversion explícita → agrupamento quantizado → ranking deterministicamente estável → palette proposta. Recolor: primeiro resolve mapping e locks (Spot, contrast, luminance, harmony), avalia colors via Engine/CMM, preview por derived override, materializa apenas Command/Apply. Comparar diferenças perceptuais com métrica declarada; não usar RGB euclidiano indiscriminadamente.

Image recolor reusa extraction. Imagens sem perfil têm tratamento explicitamente documentado; nunca atribuir ICC silenciosamente.

## Smart Measure

Engine resolve Point/Edge/Curve/Arc/Node/Bounds/Intersection; retorna snapping candidates + métricas (length, angle, radius, diameter, signed/unsigned area, perimeter) com unidade e tolerância. Área em path autointersectado precisa declarar regra de preenchimento e definição (área da região preenchida ≠ soma ingênua de shoelace para todo caso). Associative dimensions recalculam com source revision. Missing anchors → unresolved.

## Avançadas

- **Variable Width:** amostrar perfil left/right sobre arc length, reconstruir contorno expandido com join/cap/cusp treatment e self-overlap policy; preview não substitui source.
- **Pattern Editor:** avaliar fontes repetidas com grid transform, clipping e tile provenance; não flatten todo o padrão.
- **Brand Sheet:** template puro + palette/style queries → DocumentFragment validado → 1 Command de inserção, tipagem e layouts editáveis.
- **Vector Feather:** softness profile → signed-distance/edge-distance field ou máscara de coverage derivada → filtro espacial variable blur, ROI expandido; não equiparar a Gaussian Blur uniforme.
- **Perspective/Envelope:** transformação projective/envelope e adaptive subdivision para curvas e raster, com erro/tolerância e degenerações. Não confundir AffineGrid com warp de geometria.
- **True Vector Brush:** resampling + dynamics + stamped vector outlines/repeated shapes sobre spine e attachment; limites por stroke, determinismo da seed, expansão segura.
- **Mesh Gradient:** patch control points + interpolation de cor e continuidade; modelo separado, pós-v0.1-stable. Não usar enum vazio em Paint.

## Crosscutting

**Benchmarks:** tempo/p95, alocações, node explosion, ROI, memory budget, cancel latency; hardware modesto. **Property/fuzz:** degenerate paths, transformed groups, fill rules, zero lengths, clones, undo/redo, source deletion, nested clip/masks, profile errors, stability after save/load.

**Compositor:** color/effects follow standard premultiplied linear path. Previews caros usam backpressure/latest-state-wins. Import/export degradam recursos não suportados com aviso, nunca mudança silenciosa.

[Core models](#/docs/01-core/creative-features.md) · [Ferramentas](#/docs/04-ui/creative-tools-overview.md) · [Verification](#/docs/00-architecture/verification.md)