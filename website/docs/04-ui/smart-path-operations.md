# Smart Path — operações assistidas e correção vetorial

**Estado:** decisões de UX aprovadas por delegação em 2026-10-08; **especificação-alvo, não código implementado/testado**. Contratos de Core/Engine já aprovados são pré-requisitos. Este é o documento canônico de comportamento de Direct Bend, Smart Delete, Simplify, Smooth, Clean Vector e Select Same/Similar.

## 1. Contrato transversal

Cada operação fornece explicitamente: alvo/elegibilidade, scope, parâmetros em unidades e tolerâncias, hover e cursor, preview original/resultado, erros recuperáveis, confirmação/cancelamento, uma Transaction sem alterações parciais, preservação de IDs/provenance quando cabível, Undo/Redo, acessibilidade e limites de custo.

O pipeline é:
~~~text
SelectionState + ActionId
  → eligibility / scope query
  → Geometry Engine analysis and preview (same solver as commit)
  → UI overlay + status / diagnostics
  → confirm → semantic Command / prepared atomic Transaction
  → canonical Core authoring data and revised snapshot
~~~

Se `DocumentRevision` mudou desde o início da operação, recomputar ou invalidar com mensagem; nunca confirmar uma prévia baseada em referência obsoleta. Preview não altera SceneGraph. Tools não implementam algoritmo geométrico em QML/Qt.

## 2. Direct Bend — curva sem caçar handles

### Intenção

A operação Bend ativa permite clicar e arrastar diretamente um **segmento Bézier elegível**, ajustando a silhueta sem mover acidentalmente os anchors. Em Node, drag de segmento continua **sem Bend implícito**.

### Interação

1. Entrar `Vector Edit → Bend` ou ActionId `Bend Segment`; manter sub-selection válida.
2. Hover realça somente o trecho editável e mostra a localização efetiva sobre a curva; destacar explicitamente handles/nodes protegidos.
3. Down captura `PathObjectId/ContourId/SegmentRef`, parâmetro geométrico `t` determinado pelo nearest point robusto, revisão base e constraints.
4. Drag solicita ao Engine um novo par de controles `P1,P2` para produzir o deslocamento desejado próximo de `t`, com endpoints `P0,P3` fixos por padrão. Render mostra resultado e opcionalmente ghost original.
5. Up confirma uma Transaction apenas se solver achar geometria finita e compatível com constraints; Escape cancela.

**Painel contextual:** `Preserve endpoint positions` (ligado por padrão), `Preserve endpoint tangents`, `Influence/falloff`, `Strength`, `Max deviation`, `Reset preview`. Os presets não reescrevem geometry antes de confirmar.

### Casos de segurança

- Tangentes Smooth/Symmetric de nodes adjacentes são constraints, não um pretexto para mudar outros segmentos sem aviso. Se preservar continuidade exigir ajustes em neighbors, mostrar alcance antes de confirmar; opção de tratar somente o segmento pode requerer conversão explícita de NodeKind.
- Segmento `Line`: Bend solicita `Convert Line to Cubic` com preview e valor de desvio; não converter silenciosamente por hover.
- Cusp, closed contour, segmento degenerado, self-intersection ou transform singular devem receber diagnóstico específico, sem teleporte ou controles de magnitude infinita.
- Para múltiplos segmentos escolhidos, executar `Bend Individually` ou `Bend as Group` somente após operação explícita. Não aplicar vários solvers independentes em drag único sem feedback.

### Acceptance

Endpoints preservados quando policy ligada, preview/commit parity, Undo único, sem mutação no hover, caminho selecionado correto, sem NaN/Inf, limites de tempo e fallback cancelável.

## 3. Smart Delete Node — remover sem destruir a forma

**Default: Preserve Shape.** A ação principal é `Smart Delete`, e `Hard Delete / Direct Join` é alternativa explícita. Regras para um node B entre A e C:
- Se preservar silhueta for possível dentro de tolerância, fazer curve fit dos segmentos A-B e B-C para A-C; informar desvio máximo, contagem de nodes e risco topológico.
- Se o fit falhar ou mudar FillRule/topologia de modo perigoso, **não aplicar automaticamente**; mostrar resultado, comparativo e opções `Adjust tolerance`, `Hard Delete`, `Cancel`.
- Endpoints de path aberto: remoção exige regra própria de encurtamento; não criar ligação fictícia. Em contour fechado com número mínimo/área degenerada, informar impacto e impedir estados inválidos da operação (embora Core tolere degenerados representáveis).
- Node selecionado com refs de Dimension Annotation, width point, attachment ou constraint requer política de provenance: remap determinístico somente se semanticamente inequívoco; caso contrário `Needs Review` e sem substituição silenciosa de referência.
- Seleção múltipla: processar como operação global na geometria original e resolver dependências/ordem; nunca executar remoção sequencial que produza resultados dependentes da ordem de clicks.
- O report final identifica `Removed nodes`, `Preserved references`, `Unresolved dependencies` e `Max deviation`.

## 4. Simplify — menos nodes, mesma intenção visual

Ferramenta de redução de complexidade com trade-off visível entre quantidade de nodes e fidelidade da silhueta; não confundir com Merge by Distance ou Clean Vector.

**Fluxo:** selecionar Path(s) → `Simplify` → preview com `Nodes before/after`, `Maximum deviation`, `Corner preservation`, `Closed contour / holes` → ajustar tolerância em unidade documento com visualização alternativa em screen-space na zoom atual → Apply ou Cancel.

**Controles recomendados:**
- `Quality / Reduction` como preset principal legível; avançado revela tolerância explícita, preservação de Cusp, tangências e regiões de alta curvatura.
- `Protect endpoints`, `Preserve sharp corners` e `Preserve topology` ligados por padrão, sempre que tecnicamente viável.
- `Preview original outline` e mapa de nodes removidos opcionais; não exibir dezenas de overlays simultaneamente.
- `Compare` alterna visual original/resultado sem recalcular mutação autoral.
- A quantidade final de nodes não é promessa exata quando há constraints e tolerâncias.

**Garantias:** não inverte orientação por acidente (FillRule NonZero), não destrói holes silenciosamente, não cruza paths vizinhos como efeito oculto, não excede limites de work/time; operação cancelável.

## 5. Smooth — melhorar continuidade, não apenas reduzir pontos

`Smooth Path` altera tangentes/posições selecionadas conforme força e proteção de corners, sem necessariamente diminuir nodes. `Auto Smooth` é uma ação pontual de tangentes definida na [especificação de precisão](#/docs/04-ui/vector-edit-precision.md); não usá-la como sinônimo de Simplify ou de Smooth Path.

- `Smooth Selected Nodes`: aplica ajuste onde seleção parcial preserva anchors fora do escopo; indicar quais neighbors influenciam o solver.
- `Smooth Stroke`: ação por desenho/brush apropriado, pode avaliar traçado por arc length, com preservação de forma conforme limites.
- Preview permite `Strength`, `Preserve cusps`, `Maintain endpoints`, `Maximum displacement`; total de nodes pode continuar idêntico.
- Quando suavizar mudaria intencionalmente cantos protegidos, sinalizar e requerer opção explícita de relaxar constraint.
- Não criar `NodeKind::Auto` no formato persistente.

## 6. Clean Vector — diagnóstico e correção seletiva

**Não é um botão que corrige tudo sem explicação.** Primeira fase é somente análise não destrutiva do escopo selecionado.

### Diagnósticos

| Problema | Diagnóstico | Correção possível |
|---|---|---|
| Duplicated/near anchors | distância + IDs envolvidos | merge por política, apenas quando topologia permitir |
| Zero-length segment | segmentos cujo comprimento geométrico é nulo/quase nulo | remover/colapsar, sem excluir informação intencional |
| Collinear redundant node | desvio da reta/curva original | dissolve conservador |
| Micro-loop / stray contour | perímetro, área, visibilidade | remover ou manter com avaliação |
| Open contour gap | endpoints próximos e orientação | detectar/preview fechamento explícito |
| Self-intersection | localização e natureza | reportar; modificar só com ação escolhida |
| Invalid-looking tangent | descontinuidade suspeita | sugerir Smooth/Relink, nunca aplicar indiscriminadamente |

### Painel de análise

Agrupar por **Safe candidate / Needs review / Topology change**. Cada item expõe `type`, `count`, `affected objects`, `estimated visual deviation`, `reference risk`. Agrupar resultados para 10k+ nodes com virtualização, busca, filtro, ordenar por risco, selecionar no canvas e pular para ocorrência por teclado.

### Apply

- `Analyze` não muda DocumentRevision nem History.
- `Fix Selected` aplica somente issues marcados; cada correção é previamente validada no snapshot original (não deixar order-dependent behavior). Mostrar preview global, com número de nodes/contours antes/depois.
- Issues que alteram topologia/FillRule/refs requerem consentimento explícito e diagnóstico; `Fix Safe Only` nunca age sobre issues ambíguas.
- `Apply` é uma Transaction atômica; reanalisar para verificar resultados e **idempotência**: repetir `Fix Safe Only` sobre desenho já corrigido não deve criar novas mudanças sem motivo.
- `Export diagnostic report` é funcionalidade auxiliar futura; não supor implementação.

## 7. Select Same / Similar — consultas, não alterações de arte

**Ação:** abrir `Select Similar` e escolher atributo e escopo; default preserva contexto (page/group/visible-editable).

**Categorias:** fill/stroke (cor exata e diferença perceptual com métrica declarada), width, dash, opacity, blend mode, vector type, TextStyle (família/peso/tamanho), linked Style/Swatch, Symbol, Effect, Asset Resource, Appearance. Combinar filtros `AND/OR` com preview de contagem, evitar precisão falsa.

**Fronteiras:** locked podem ser mostrados em resultados para inspeção por opção explícita, nunca entrar em alteração posterior silenciosa; hidden ficam fora por padrão. Uma seleção mista de cores/propriedades deve pedir atributo-base explícito.

**Resultado:** Replace/Add/Subtract/Invert within scope operam apenas em SelectionState; não criam Undo autoral nem sujam documento. Navegação por teclado, contagem de itens, modos alto contraste e comando `Locate next match` são obrigatórios para acessibilidade.

## 8. Feedback, UI e assistência

Operações avançadas aparecem em menu contextual/overflow e Command Palette; a context bar evidencia a operação corrente e os principais 1–3 parâmetros. Não criar um botão de toolbar fixa para cada suboperação.

Toda ação informa: `What will change`, `What stays protected`, `Scope`, `Preview`, `Apply/Cancel`. Antes/depois pode usar ghost ou comparação alternável, sem obrigar split-view. Para erros, manter causa e correção no painel, não gerar toast por candidato. Jobs caros mostram progresso, cancelamento e `latest preview wins`.

Configuração simplificada exibe presets e números essenciais; detalhes avançados permanecem disponíveis em disclosure sem alterar a semântica. Expor descrição para termos como tolerance, topology, FillRule e refit na primeira utilização.

## 9. Testes e critérios transversais

- Documentos reais: arte com paths de 10k+ nodes, desenho minimalista, SVG importado, logos, loops/holes, illustrations complexas, strokes e transforms aninhados.
- Property: geometria finita, identidade preservada quando não removida, snapping não implica autoral mutation, Undo/Redo restauram exactly previous revision.
- Degenerados: nearly coincident, cusp aguda, zero-length, self-intersection, Line/Cubic mixed, closed contours, masks/clip/locked.
- Atomicidade: aplicar lote com uma correção inválida não deixa apenas as válidas alteradas sem consentimento.
- A11y: ActionIds, roteamento por teclado, feedback textual e foco persistente em análise longa.
- Performance: medir p95 por operação, memória/tempo de análise, cancel latency, preview backpressure e contagem de nodes vs custo.

## Referências

[Smart Path overview](#/docs/04-ui/smart-path.md) · [Vector Edit precision](#/docs/04-ui/vector-edit-precision.md) · [Geometry](#/docs/02-engine/geometry.md) · [Creative operations Engine](#/docs/02-engine/creative-operations.md) · [Verification](#/docs/00-architecture/verification.md)

Exemplos funcionais: [Illustrator — Refine paths](https://helpx.adobe.com/illustrator/desktop/draw-shapes-and-paths/modify-paths/refine-paths.html), [Figma — Vector edit](https://help.figma.com/hc/en-us/articles/360039957634-Edit-vector-layers).
