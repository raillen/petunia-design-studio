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
| 7 | Gradient | Stops múltiplos, radial/cônico, snap de ângulo, preview vivo | Médio, sem modelo novo |
| 8 | Picker + Measure | Amostra stroke/gradiente/appearance; Measure com área | Pequenos, mesmo padrão |
| 9 | ShapeBuilder | Síntese booleana de região com highlight + 1 undo | Médio, usa boolean avaliado |
| 10 | Transparency como modificador | `TransparentGradient` vira 2º `ModifierKind` (gatilho de revisita do ADR: generaliza identidade/ordem) | Médio-grande, valida a fundação |
| 11 | Text-on-Path | Handles start/end, fluxo no path | Médio, spec 10.6 V1 |
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
