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
