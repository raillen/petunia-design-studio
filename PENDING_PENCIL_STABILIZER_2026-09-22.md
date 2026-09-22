# Futuro — Estabilizador Procreate-like do Pencil (opção 3)

- **Branch:** `refactor/tools-funcionamento-2026-09-22`
- **Status:** PLANEJADO (Batch 3 fechou V1 + Sculpt; isto é o passo seguinte)
- **Decisão:** Batch 3 = Pencil V1 + Sculpt. Opção 3 fica para o futuro.

## Por que ficou para depois

Estabilizador ao vivo (estilo StreamLine) muda o gesto de `Down/Move/Up`
para um loop com atraso e média móvel. É outro estado, outro teste,
outro ajuste de UI (slider). Não cabe no V1 sem estourar o escopo.

## O que implementar quando chegar a hora

1. `PencilStabilizer { amount: f64, pressure_flow: f64 }` com média móvel
   sobre as amostras (Procreate: `Stabilization` + `StreamLine`).
2. Correção por velocidade (Clip Studio): mais estabilização devagar,
   menos estabilização rápido (evita atraso do traço).
3. Pós-correção configurável: força do `smooth_samples` após soltar
   (hoje fixo em `1.5, 1` no modo Equilibrado).
4. Slider na toolbar contextual, espelho do padrão `marquee_rule`
   (commit `f717540`): propriedade + callback + sync.
5. Pressão só entra aqui se o modelo de stroke suportar largura variável
   (hoje `StrokeItem.width` é escalar — ver `appearance.rs:532`).

## Critério de aceite do futuro

Mesmos testes do Batch 3 verdes + teste novo: traço tremido de entrada
gera caminho com menos vértices e desvio limitado, com e sem velocidade.
