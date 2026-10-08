# Seleção, nodes e handles — Smart Path

**Status:** **proposta detalhada para discussão conjunta de UX**, não decisão final sobre indicadores/gestos. O **modelo híbrido Select + Vector Edit** e a separação entre seleção e mutação já são decisões aceitas no [ADR-0011](#/docs/00-architecture/adr/0011-hybrid-vector-edit.md).

Esta página especifica a próxima camada: seleção de objetos, hit-test, sub-selection, handles, gestos e controles contextuais. Não representa funcionalidade já codificada.

## Referências

| Produto | Padrão que vale aproveitar |
|---|---|
| [Illustrator — preferências de anchors/handles](https://helpx.adobe.com/illustrator/desktop/draw-shapes-and-paths/learn-drawing-basics/adjust-anchor-point-handle-and-bounding-box-display-size.html) | tamanho de indicadores, highlight no hover e mostrar/ocultar handles múltiplos |
| [Figma — Vector Edit](https://help.figma.com/hc/en-us/articles/360039957634-Edit-vector-layers) | edição multipath, bounding box em nodes selecionados, Node/Bend/Cut/Width dentro do contexto |
| [Affinity — Node Tool](https://s3-eu-west-1.amazonaws.com/affinity-docs/help/designer/en-US.lproj/pages/Tools/tools_node.html) | conversão Cusp/Smooth, join/break e snapping contextual |
| [Inkscape — edição avançada](https://inkscape.org/pt/doc/advanced/tutorial-advanced.pt_BR.html) | seleção de nodes por clique/marquee e arrasto de segmentos |

São referências de comportamento, não especificação pixel-perfect nem importação de um modelo de documento externo.

## 1. Três estados que não podem ser confundidos

**Hover**: alvo sob cursor, nunca muta seleção ou documento.

**Selection**: conjunto de ObjectIds ou NodeIds sobre os quais a próxima ação pode atuar; é Session State.

**Focus**: destino atual do teclado e da tecnologia assistiva; pode apontar para panel, campo, canvas ou objeto sem selecioná-lo.

**Active interaction** é uma quarta dimensão: pointer capture + preview transient em andamento. Visualmente deve ser reconhecível, mas não é sinônimo de selection.

~~~text
Hover Target ≠ Selected Target ≠ Keyboard Focus ≠ Captured Operation
~~~

O contexto decide quais operações estão disponíveis, sem inferir intenção destrutiva a partir do cursor.

## 2. Select — interação de objetos

### Clique e seleção

- Click em conteúdo de objeto elegível seleciona o objeto mais alto no z-order relevante.
- Click vazio limpa ObjectSelection quando não há modificador de seleção.
- Click no objeto já selecionado mantém selection; drag sobre ele prepara MovePreview.
- Click em locked/hidden não seleciona para edição, respeitando policy de hit-test e possibilidade de inspecionar via Layers.
- Group tem scope próprio; duplo clique entra em isolamento, não seleciona filho arbitrário fora de contexto.
- Outline-only selection é uma preferência possível; o default proposto é selecionar pela área preenchida e pelo stroke visível.

### Objetos sobrepostos

Definir **Cycle Overlapping Selection** como ActionId, com lista contextual opcional contendo nome, tipo, layer e lock/visibility. Não exigir Alt+Click como única rota: alguns WMs Linux o reservam para mover janela.

O ciclo respeita o scope ativo e z-order. Ao abrir lista, não altera DocumentRevision.

### Seleção por área

Marquee a partir do espaço vazio não move o objeto.

**Duas políticas candidatas para fechar com o usuário:**

1. **Contenção simples:** seleciona objetos completamente dentro do retângulo, independente do sentido.
2. **Direcional (estilo CAD):** esquerda→direita = inteiramente dentro; direita→esquerda = crossing/intersectando.

Lasso é ação explícita para contorno irregular. A política visual de preenchimento da marquee, cor e alvos por categoria ainda exige validação.

### Multi-selection

Adicionar/remover seleção por Action/Modifier nomeado, sem atalhos exclusivos hardcoded. Multi-selection mantém ordem determinística e âncora ativa para inspectors, alinhamento e transform.

Mover um item da seleção **não** perde os demais, salvo click explícito que altere seleção. Esse comportamento deve ser idêntico em Mouse, Stylus e teclado.

## 3. Vector Edit — sub-selection

### Hierarquia

~~~text
Object Selection
  └── PathObject(s)
        └── ContourId
              ├── NodeId(s)
              ├── SegmentRef(s)
              └── HandleRef(s)
~~~

`NodeId` é identidade autoral estável. `SegmentRef` e `HandleRef` são referências locais tipadas e ligadas à revision/adjacência; não assumir que índice em Vec é identidade persistente.

### Clique

- Click em node (Node operation): seleciona NodeId.
- Click em segmento (Node operation): torna segmento alvo de inspeção/seleção contextual, **não** inicia Bend.
- Click em handle visível: seleciona/ajusta o handle correspondente, dentro do modo Node.
- Click vazio dentro de Vector Edit: limpa sub-selection, mas não sai do contexto e não apaga ObjectSelection.
- Shift+Click é candidato de gesto multiplataforma para adicionar/remover nodes; atalho final e casos com caneta ficam para decisão conjunta.

### Arrasto quando vários nodes estão selecionados

Política recomendada:

1. Down sobre **node já selecionado** → mover a sub-selection inteira com delta comum;
2. Down sobre **node não selecionado** sem modificador → substituir seleção e mover só esse node;
3. Down com modificador de add/toggle → alterar seleção, sem drag simultâneo até intenção desambiguada;
4. Down em handle → modifica handle, **não** translada todos os nodes selecionados;
5. Down em segmento → o comportamento depende de Node/Segment Select/Bend; Node nunca aplica Bend implícito.

A seleção deve permanecer estável durante drag capturado. Hit-test de outros nodes não pode roubar o gesto.

### Marquee e Lasso de nodes

Confinados a paths editáveis no contexto atual. Quando selecting multiple paths, a mesma marquee pode incluir nodes de objetos distintos.

A seleção usa DocumentPoint→ViewPoint para consultar candidatos via Engine, mas visualiza overlay em logical pixels. Não projetar todo o hit-test em pixels de dispositivos sem DPR consistente.

### Transformação de nodes

Selecionar **2 ou mais nodes** pode habilitar bounding box de sub-selection com Move/Scale/Rotate contextual, como no Figma. Ela não deve aparecer por default sobre um único node.

Transformar nodes não transforma o SceneNode inteiro; delta é convertido para o espaço local de cada path, preservando IDs, handles e constraints. Para bounding degenerada (todos em linha ou mesmo ponto), desabilitar eixo impossível em vez de dividir por zero.

Oferecer Transform Separately como ação avançada posterior, não default surpresa.

## 4. Semântica dos nodes

| Tipo | Significado | Manipulação |
|---|---|---|
| **Cusp** | tangentes independentes, descontinuidade angular permitida | arrastar um handle não move o outro |
| **Smooth** | tangentes colineares; comprimentos independentes | direção do handle oposto acompanha, comprimento preservado quando possível |
| **Symmetric** | tangentes colineares e mesma magnitude | arrastar um handle espelha ângulo e comprimento |

Para path aberto, só o handle pertinente existe nas extremidades. Não fabricar handle oposto inexistente apenas para mostrar simetria.

Converter tipos de node deve procurar preservar a forma dentro de tolerância; quando a geometria obrigatoriamente muda, preview mostra o resultado antes do Commit.

### Formas dos indicadores — proposta inicial

- **Cusp**: quadrado.
- **Smooth**: círculo.
- **Symmetric**: losango/diamante.
- **Endpoint**: mantém tipo geométrico, mas recebe sinalização extra de início/fim (por exemplo pequeno indicador de direção em hover/seleção), para não misturar topologia com continuidade.
- **Active / Focus**: usa anel/outline adicional; não troca o tipo de node.

Cores exatas são tokens temáticos, não cores rígidas desta especificação. Estados não dependem exclusivamente de cor.

## 5. Handles — visibilidade progressiva

**Default recomendado:**

- Nodes pertencentes aos paths editáveis aparecem com marcadores discretos.
- Handles dos nodes selecionados aparecem.
- Quando muitos nodes estão selecionados, fornecer toggle **Mostrar handles dos selecionados**.
- Fornecer opção **Mostrar todos os handles**, adequada a usuários que desejam comparação visual entre tangentes.
- Handles de nodes não selecionados não precisam dominar a tela por default.
- Zoom extremo pode simplificar linhas auxiliares de nodes distantes do foco, sem ocultar targets selecionados ou ações de acessibilidade.

Essas políticas refletem a configurabilidade do Illustrator sem obrigar todos ao mesmo nível de informação.

### Hit target ≠ tamanho visual

O indicador pode ser discreto e o alvo clicável maior. Ambos respeitam escala da UI, device type, zoom e acessibilidade.

O Engine compara candidatos em **logical screen-space** após conversão explícita de space; o threshold é configurável e mensurável, não um EPSILON geométrico do documento.

Não congelar valores fixos de 6, 8 ou 12 px sem testes em DPI/monitor/caneta e com acessibilidade.

## 6. Precedência de hit-test

A política de escolha deve ser única, inspecionável e testada:

| Contexto | Preferência |
|---|---|
| Select | transform handle explicitamente ativo → objeto preenchido/stroke elegível pelo z-order → vazio |
| Node | handle de node selecionado → node selecionado → node não selecionado → segment/ref → vazio |
| Bend | node/handle protegido → segmento alvo de Bend → vazio |
| Width | width handle/point → spine elegível → vazio |
| Pen | endpoint para continuar/fechar → snap candidate válido → novo node |

Ordenar por **intenção da operação, elegibilidade, alvo específico, distância em screen-space e z-order**. Quando há múltiplos candidatos equivalentes, oferecer ciclo/disambiguation, não escolher silenciosamente um node “errado”.

Quando um node e um handle se sobrepõem, a precedência não pode tornar impossível alcançar um deles: oferecer alternância de alvo ou ocultação contextual de handles.

## 7. Snapping sem tremer

Preview deve comunicar quando está conectado a node, extrema, midpoint, grid, guide, alignment ou intersection.

Histerese evita que o snap salte continuamente entre dois candidatos próximos: manter alvo escolhido até outro superar um limiar adequado. Aplicar ranking estável por classe, proximidade e scope.

Expor **temporarily disable snapping** e toggles por classe através de Actions; não assumir Alt como único modificador no Linux. O candidato é estado derivado, não Document.

## 8. Context bar — três níveis

A barra contextual precisa evoluir conforme seleção, **sem mudar brutalmente de layout ao passar o cursor**.

**Select:** objeto(s), tamanho/posição/rotação, align/distribute, group, edit.

**Vector / nenhum node:** operação ativa (Node/Bend/Cut...), path-level actions, snapping, opção de visibilidade de handles.

**Vector / node(s) selecionado(s):** Cusp/Smooth/Symmetric, Split/Join/Close quando válidos, posição X/Y, multiplicidade, seleção invertida, ações de Simplify/Smart Delete.

**Vector / segmento selecionado:** Line/Cubic/Segment operation, conversão, ponto e comandos vinculados à operação ativa.

Os controles habilitados/desabilitados mostram causa; campos numéricos permitem digitação explícita, keyboard stepping e drag-scrub sensível, com commit controlado. Input truncado ou inválido **nunca** escreve coordenada incompleta no Document. Não alterar layout por hover; detalhes de tooltips podem ser revelados progressivamente.

## 9. Feedback visual

| Estado | Sinal proposto |
|---|---|
| Normal | node com forma semântica e baixo destaque |
| Hover | realce de borda, tooltip curto só se útil |
| Selected | fill/borda mais fortes; handles relevantes |
| Focused | anel de foco distinto de selected |
| Captured/Dragging | ghost opcional, feedback de delta e snap |
| Locked/Disabled | aparência/informação de motivo sem permitir edição |
| Unresolved/Invalid | aviso contextual recuperável; sem mutação |

Evitar toast a cada click, node move ou seleção. Aviso inline durante gesto, erros significativos por notificação persistente/diálogo apenas quando realmente bloquear o trabalho.

As diferenças de estado devem funcionar em modo claro/escuro, alto contraste e reduced-motion. Não basear comportamento exclusivamente em hover.

## 10. Semântica de Undo, Preview e erros

1. Pointer Down captura revisão base e alvo estável.
2. Move atualiza transient geometry sem mutar Document.
3. Pointer Up confirma **uma única Transaction** se houver alteração autoral real.
4. Escape ou perda legítima de captura cancela preview, sem HistoryEntry.
5. Mudança concorrente de revisão invalida ou recalcula preview de forma explícita.
6. Click, hover, mudança de SelectionState, viewport e toolbox não deixam documento dirty.
7. Transform de multi-node é atômico para todos os objetos afetados.
8. Um handle inválido / constrained transform impossível devolve erro tipado; nunca deixa path parcialmente mutado.

## 11. Acessibilidade e múltiplas formas de entrada

O canvas apresenta uma árvore semântica navegável para tecnologias assistivas (paths → contours → nodes) com ActionIds para selecionar, alterar tipo de node, mover em incrementos e revelar handles.

Ações como Smart Delete e Clean Vector precisam de configuração/feedback textual e via teclado, sem exigir distinguir cores ou pequenos handles.

Pointer device e pressure não mudam o significado de seleção por si só. Preferências para tamanho de targets, foco, densidade de overlays, velocidade de drag e incremento de nudges devem ser controláveis.

## 12. Quality gates

- Single click / double click / drag threshold não causa moves falsos.
- Node eleito sob zoom extremo / canvas rotation / DPR alto corresponde ao alvo real.
- Selected handle sobreposto a node mantém rota de desambiguação.
- Multi-node drag move todos e só os nodes esperados.
- Click em node não selecionado não move seleção antiga.
- Troca Node↔Bend preserva sub-selection.
- Selected-only vs show-all handles não muda geometria/History.
- Marquee crossing/containment seguem a policy escolhida.
- Bounds degenerado não produz NaN/div-by-zero.
- Cusp/Smooth/Symmetric respeitam tangent constraints e node identity.
- Locked/mixed selection não faz edit parcial silencioso.
- Error/revision conflict não gera commit parcial.
- Screen reader/focus navega sem mouse.

## 13. Pontos de decisão para revisão conjunta

**A. Marquee em Select:** contenção uniforme ou seleção direcional (inside/crossing)?

**B. Transformação de subseleção:** bounding box aparece automaticamente com 2+ nodes selecionados, ou somente quando ativada ação Transform Nodes?

**C. Handles:** default selecionados + opção show-all, ou todos os handles sempre visíveis em Vector Edit?

Outras preferências — atalhos, paleta, tamanho de anchors, scroll/drag-scrub, escolha de iconografia — pertencem à futura etapa de UX detalhada.

[Vector Edit](#/docs/04-ui/vector-edit-interaction.md) · [Smart Path](#/docs/04-ui/smart-path.md) · [Acessibilidade](#/docs/04-ui/accessibility.md) · [ADR-0011](#/docs/00-architecture/adr/0011-hybrid-vector-edit.md)
