# Diário de Decisões — Ferramentas (09.31 na prática)

Log append-only. Cada batch registra: decisão, referência de mercado, porquê.
Doutrina-mãe: ADR 09.31 (`petunia-design-studio/09 31 — Non-Destructive Editing EffectChain ADR.md`).

## Doutrina operacional (vale para todas as ferramentas)

- **Edição de base vs. modificador.** Criar traço, mover nó, deletar nó,
  fatiar path: edições topológicas da fonte — a fonte É o path, nada se
  preserva além dela, undo cobre. Transformação paramétrica (offset, cantos
  vivos, warp futuro): modificador vivo, fonte intacta, Bake explícito.
- **Um gesto, um undo (F-01).** `Move` prevê, `Up` commita via `submit_all`.
  Sem escrita por evento.
- **Conversão implícita só dentro do gesto que a exige.** Faca corta qualquer
  forma: `ConvertToCurves` entra no mesmo batch do corte (o gesto pede corte,
  não existe corte sem curva). Fora de gesto, conversão continua explícita.
- **Stubs são silenciosos por construção antiga;** todo stub tocado ganha
  comportamento ou registro de futuro — nunca `todo!()`.

## Roadmap decidido (ordem de execução)

| # | Batch | Escopo | Motivo da ordem |
|---|---|---|---|
| 6 | Knife × Scissors | Linha de corte amostrada em 1 undo; tesoura por clique; auto-convert no batch | Pequeno, fecha a família 10.2 |
| 7 | Gradient | Stops múltiplos, radial, snap de ângulo, preview vivo | Médio, sem modelo novo |

## Batch 7 — Gradient (decisões)

- **Linear + Radial, sem cônico.** `Paint` não tem `ConicGradient` e SVG/PDF não
  o representam — modelo novo + fallback de export é outro batch. Registrado futuro.
- **Stops no canvas:** duplo-clique na linha adiciona (cor amostrada do ponto
  médio em literal `rgb()`), duplo-clique no stop remove (mínimo 2), arrastar
  move offset (Shift = passos de 0.05). Overlay `GradientOverlay` novo expõe
  linha + alças (aditivo em `CanvasOverlays`, sem quebrar leitores).
- **Sem cor inventada:** criação a partir de sólido duplica a cor nos 2 stops;
  troca de geometria preserva stops; `sort_and_reindex` mantém ids estáveis.
- **Clique nunca cria** (limiar 3 px). Shift trava vetor em 45°. Um gesto, um undo.
- **Transparency intocado:** continua proxy documentado até virar modificador (batch 10).
| 8 | Picker + Measure | Amostra stroke/gradiente/appearance; Measure com área | Pequenos, mesmo padrão |

## Batch 8 — Picker + Measure (decisões)

- **Color = par completo.** Amostra fill primário (sólido ou gradiente preservado)
  + stroke (paint + largura) e aplica via um `SetAppearance` por alvo: preserva
  resto da stack, 1 undo. Style continua stack inteira. Sem split por modificador
  (previsível > esperto).
- **Travados nunca amostram** (filtro `visible && !locked` — faltava).
- **Measure com área:** `MeasureMode::{Distance, Area}`; arrasto = retângulo
  (área + perímetro, overlay marquee); parado com seleção = soma das áreas
  avaliadas (`measured_area`, respeita modificadores vivos).
- **Infra:** `AreaReadout`/`area_readout` na geometria (Table B) para a UI ler.

## Nota de infra (descoberta nos batches 7–8)

- O `target-dir` compartilhado entre worktrees envenena fingerprints quando dois
  agentes compilam juntos (sintomas: itens "inexistentes" que existem).
  **Regra: `CARGO_TARGET_DIR=/tmp/petunia-tools-target` em todo comando cargo
  desta worktree.**
| 9 | ShapeBuilder | Síntese booleana de região com highlight + 1 undo | Médio, usa boolean avaliado |

## Batch 9 — ShapeBuilder + SmartFill (decisões)

- **Região = face por assinatura de cobertura** (interseção de quem cobre menos
  união do resto), sobre outlines avaliados. Para 2 formas = faces exatas do
  Illustrator; para N, aproximação documentada (faces distintas podem partilhar
  assinatura). Flood de espaço negativo aberto fica futuro (face detection).
- **Click cria, Alt subtrai, drag funde.** Subtração que consome tudo deleta o
  objeto; fontes do merge são preservadas (desvio documentado do AI, que
  consome — reversível via undo, sem perda silenciosa).
- **SmartFill = mesmo motor, fill default** (`ptnd.blue/500`, precedente do
  gradient). Flood real fica futuro.
- **Achado:** união booleana com acumulador vazio retornava vazio para sempre
  (identidade quebrada) — corrigido com teste.
- **Overlay:** `region_preview` (doc-space) para highlight de hover/drag.
| 10 | Transparency como modificador | `TransparentGradient` vira 2º `ModifierKind` (gatilho de revisita do ADR: generaliza identidade/ordem) | Médio-grande, valida a fundação |

## Batch 10 — TransparentGradient, 2º modificador (decisões)

- **Modelo:** `TransparentGradient { start, end, stops: OpacityStop[] }`;
  avaliação no domínio opacidade (`evaluate_opacity_at`, produto das entradas),
  geometria intacta. Domínios independentes: contour e transparency coexistem
  sem vazar um no outro.
- **Ferramenta:** arrasto define o vetor (Shift 45°), default opaque→transparent;
  substitui o proxy `SetStackOpacity`. Clique nunca cria.
- **Render honesto:** sem infra de máscara, export/preview amostram o centro
  (`sampled_opacity`, documentado); máscara cheia fica futuro.
- **Bake explícito:** `bake_transparency` achata a amostra central na opacidade
  base (documentado com perda) + `bake_contour` já existia.
- **API de cadeia (futura UI de lista):** enable/disable/remove/move-to-front via
  `SetModifiers`, NoOp sem entrada no histórico (F-22).
- **Check de revisita do ADR 09.31:** generalização OK (identidade por `id`,
  ordem = ordem do vec, custo linear); falta só UI de lista/ordem.
| 11 | Text-on-Path | Handles start/end, fluxo no path | Médio, spec 10.6 V1 |

## Batch 11 — Text-on-Path (decisões)

- **Modelo:** `ShapeKind::Text.on_path: Option<TextOnPathAttachment { target, start, end }>`
  (`serde default`: arquivos v1 abrem). Sem `ToolKind` novo — a Text tool cria ao
  clicar/arrastar sobre um path (Illustrator). O path nunca é consumido.
- **Geometria:** `GPath::{outline_length, sample_at, nearest_t}` por comprimento
  de arco achatado (F-21; exato-em-curva POST_V1). Span degenerado ganha mínimo 0.01.
- **Gestos:** clique cria start=t..1.0; drag define start..end; handles arrastam
  com 1 undo; Alt-clique/detach volta a reto. Bounds do texto = span.
- **Export honesto:** SVG emite `<text><textPath href="#target">` (referência por id,
  path d embutido em comentário); PDF registra `TEXT_ON_PATH_FLATTENED` e cai no
  fluxo de bounds. Preview curvo de glifos (shaping) fica futuro.
- **Regressão:** clique longe do path continua criando headline 160×32.
| 12 | SmartFill real | Flood de regiões limitadas (face detection — spike antes) | Pesquisa antes |
| 13 | Seleção raster (epic) | Modelo de máscara + marquee/lasso/brush/flood + ops | Infra nova |
| 14 | Warp/Perspective (epic) | 3º modificador, 10.8 V1_REQUIRED | Infra + matemática |
| — | Photo paint/retouch, Vector Brush, Place Image, Vector Crop, Stroke Width, Line | Batches avulsos por demanda | Pequenos-médios |

## Batch 6 — Knife × Scissors (decisões)

- **Knife = linha de corte (Corel/Affinity).** A linha do arrasto vira peças de
  verdade via `geometry::cut_path_by_line` (Sutherland–Hodgman por semiplano,
  fechado fica fechado), tudo num único `submit_all`. Primeira tentativa
  (amostrar a linha e furar pontos) foi descartada: `slice_path` só move a
  costura, corte invisível.
- **Scissors = clique que parte (Illustrator).** Down+Up sem arrasto =
  `split_path_at_point`: loop abre em 1 peça, stroke aberto divide em 2.
- **Alvos:** proximidade ao contorno avaliado + `hit_test`, topmost-first,
  visível/destravado; paramétricos auto-convertem no batch, texto e
  containers excluídos (doutrina: conversão dentro do gesto).
- **Achado registrado:** `hit_test` nunca acerta path aberto (sem interior) —
  Select também não clica strokes abertos. Fix global de hit stroke-aware fica
  futuro; a faca resolve no escopo com `near_object`.
- **Sem fishermen:** peças degeneradas (<1pt de comprimento) caem; sem remoção
  de micro-fragmentos (10.2: sem deleção silenciosa).
- **Overlay distinto:** Knife mostra a linha; Scissors mostra o ponto de corte.
