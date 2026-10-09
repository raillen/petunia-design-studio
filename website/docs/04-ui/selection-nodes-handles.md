# Seleção, nodes e handles — Smart Path

**Status — decisões de UX aprovadas em 2026-10-08:** modelo híbrido Select + Vector Edit ([ADR-0011](#/docs/00-architecture/adr/0011-hybrid-vector-edit.md)); tipos e indicadores Cusp/Smooth/Symmetric; visibilidade progressiva de handles; seleção Marquee por contenção como padrão, Interseção por ação e modo direcional opcional; Lasso contextual; transformação de 2+ nodes com bounding box discreta e desativável; desambiguação de alvos por candidatos; Shift para adicionar/remover seleção; navegação semântica por teclado e movimentos incrementais configuráveis. **Detalhamento aprovado em páginas canônicas:** tangentes/constraints e snapping estão em [Vector Edit — precisão](#/docs/04-ui/vector-edit-precision.md). **Ainda refináveis:** prioridade fina para candidatos exatamente coincidentes, design final de overlays e context bar, presets visuais e atalhos secundários. **Não implementado/testado:** a aprovação documenta comportamento-alvo, não funcionalidade presente no código.

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

### Seleção por área — política aprovada

Marquee começa em espaço vazio elegível do contexto ativo; o gesto não move nenhum objeto. Seleção por área é **Session State**, não Command autoral.

1. **Padrão: Contenção**, independente da direção do arrasto. Para objetos, incluir somente quando a geometria selecionável efetiva está inteiramente no retângulo de seleção em coordenadas de tela, respeitando transformações, clipping e escopo; não assumir que um bounding box axis-aligned é a própria geometria.
2. **Interseção (crossing):** disponível como ActionId e escolha de política na barra contextual; inclui também objetos cuja geometria selecionável toca/cruza a área.
3. **Preferência avançada: Direcional**, opt-in. Esquerda→direita usa Contenção; direita→esquerda usa Interseção. A direção só determina a política no preset direcional; não modifica o comportamento padrão.
4. **Lasso contextual:** ativação explícita por ActionId/controle contextual para região livre. Deve permitir substituir, adicionar ou subtrair seleção com rotas por modificadores configuráveis e UI acessível. Usar a mesma semântica de Contenção/Interseção, conforme a política ativa, sem selecionar itens fora do contexto de edição.
5. Em **Vector Edit**, Marquee/Lasso testam o **centro geométrico do anchor/node** projetado em viewport, não sua área clicável ampliada. Handles e segmentos não passam a ser selecionados incidentalmente por essa regra: precisam de ação/escopo próprios.

Durante o gesto, mostrar preview reversível da área e dos candidatos; ao soltar, atualizar apenas SelectionState. Escape restaura a seleção anterior. Para objetos parcialmente clipados, a query considera sua porção selecionável/visível conforme o escopo; visibilidade e lock continuam determinantes. A aparência final de region fill/outline é item de GUI pendente.

### Multi-selection e modificadores — decisão aprovada

- **Shift + clique** alterna inclusão/remoção do alvo elegível, tanto em Select como em Vector Edit. O usuário pode remapear atalhos; menus, barra contextual, comando e teclado oferecem rota equivalente sem Shift.
- Clique simples substitui seleção quando atingir alvo não selecionado; clique sobre item já selecionado preserva a seleção. Arrastar item já selecionado prepara deslocamento do conjunto, não reduz a seleção acidentalmente.
- Ações de **Select Inside**, **Select Behind / Cycle Overlapping** e navegação profunda em groups têm ActionIds e alternativas visíveis/configuráveis. Não depender exclusivamente de Alt, reservado por vários gerenciadores de janelas Linux.
- Marquee/Lasso suportam Replace/Add/Subtract por ações e modificadores configuráveis. Alternar a política de seleção não deve modificar silenciosamente o conjunto já selecionado.
- **Select All / Invert Selection** operam dentro de um escopo informado (layer, group, paths editáveis etc.), respeitando locks e filtros. Inverter seleção não expande para documento inteiro sem o usuário conhecer o escopo.
- O conjunto de seleção mantém ordem determinística e âncora ativa para inspectors, alinhamento e transformação. Mouse, caneta e teclado produzem a mesma semântica; pressure não atua como modificador de seleção.

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
- Shift+Click alterna inclusão/remoção de nodes (decisão aprovada); caneta, teclado e tecnologias assistivas possuem ações equivalentes, sem necessidade de reproduzir o modificador físico.

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

### Transformação de nodes — decisão aprovada

Ao selecionar **2 ou mais nodes distintos**, apresentar automaticamente uma bounding box **discreta** com ações de Move/Scale/Rotate da subseleção, sem encobrir markers/handles nem roubar sua precedência de hit-test. Preferência visível permite desativar a bounding box automática; a ação **Transform Nodes** permanece disponível. Com um único node, não mostrar bounding box automaticamente.

Transformar nodes não transforma o SceneNode inteiro. Delta de movimento/rotação/escala deve ser convertido ao espaço local de cada PathObject, preservando NodeId/ContourId e a integridade de constraints. Se a subseleção tem bounds degenerados (nodes colineares/coincidentes), oferecer somente graus de liberdade matematicamente válidos; não dividir por zero nem causar salto geométrico.

Preview não modifica geometria autoral; confirmar múltiplos paths realiza uma Transaction atômica. **Transform Separately** pode surgir como comando avançado posterior e nunca como comportamento implícito.

## 4. Semântica dos nodes

| Tipo | Significado | Manipulação |
|---|---|---|
| **Cusp** | tangentes independentes, descontinuidade angular permitida | arrastar um handle não move o outro |
| **Smooth** | tangentes colineares; comprimentos independentes | direção do handle oposto acompanha, comprimento preservado quando possível |
| **Symmetric** | tangentes colineares e mesma magnitude | arrastar um handle espelha ângulo e comprimento |

Para path aberto, só o handle pertinente existe nas extremidades. Não fabricar handle oposto inexistente apenas para mostrar simetria.

Converter tipos de node deve procurar preservar a forma dentro de tolerância; quando a geometria obrigatoriamente muda, preview mostra o resultado antes do Commit.

### Formas dos indicadores — tipos aprovados

- **Cusp**: quadrado.
- **Smooth**: círculo.
- **Symmetric**: losango/diamante.
- **Endpoint (detalhe ainda proposto)**: mantém tipo geométrico, mas pode receber sinalização extra de início/fim (por exemplo pequeno indicador de direção em hover/seleção), para não misturar topologia com continuidade.
- **Active / Focus**: usa anel/outline adicional; não troca o tipo de node.

Cores exatas são tokens temáticos, não cores rígidas desta especificação. Estados não dependem exclusivamente de cor.

## 5. Handles — visibilidade progressiva

**Política aprovada:**

- Nodes pertencentes aos paths editáveis aparecem com marcadores discretos.
- Handles dos nodes selecionados aparecem por padrão, inclusive quando vários nodes estão selecionados.
- Oferecer opção **Mostrar todos os handles** para comparação visual de tangentes.
- Evitar poluição visual com handles de nodes não selecionados por padrão.
- Zoom extremo pode simplificar linhas auxiliares distantes, sem ocultar targets selecionados ou rotas de acessibilidade.
- Um controle para reduzir/ocultar handles em multisseleção é uma **preferência opcional ainda a detalhar**, não um motivo para contrariar a visibilidade aprovada.

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
| Node | node/handle elegíveis → segment/ref → vazio; **ordem fina de node versus handle sobrepostos segue aberta** e deve ter desambiguação |
| Bend | node/handle protegido → segmento alvo de Bend → vazio |
| Width | width handle/point → spine elegível → vazio |
| Pen | endpoint para continuar/fechar → snap candidate válido → novo node |

### Candidatos de hit-test e desambiguação — decisão aprovada

A query do Engine produz uma **lista estável de alvos elegíveis**, classificada por intenção da operação, elegibilidade, especificidade, proximidade em logical screen-space, selection/active state e z-order quando aplicável. A UI mostra o candidato ativo sem alterar Document nem SelectionState apenas por hover.

Se dois nodes, ou node e handle, coincidirem ou competirem no mesmo hit region, oferecer **Cycle Overlapping Target** por ActionId/atalho configurável e um seletor contextual sob demanda com tipo, nome, path e estado locked/hidden quando informativo. Para caneta e toque, prever equivalente contextual acessível, sem exigir Alt nem gestos de precisão impossíveis. Uma seleção via Layers/árvore semântica também deve alcançar todos os candidatos. Fechar o seletor sem escolher não muda seleção.

A **precedência exata entre node e handle perfeitamente coincidentes** ainda exige avaliação conjunta e testes de usabilidade; não congelar uma ordem rígida arbitrária. Independentemente dela, todo alvo elegível deve continuar selecionável. Separar seleção de alvo, foco e captura do gesto: uma vez capturado, outro candidato não rouba o pointer.

A hierarquia de contextos, locks, revisão e mudança de target durante o ciclo seguem o contrato de Session/Input. Distância usa tolerâncias configuráveis de tela, não tolerâncias de geometria autoral.

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

## 11. Acessibilidade e múltiplas formas de entrada — decisão aprovada

O canvas oferece uma **árvore semântica navegável** por PathObject → ContourId → NodeId → HandleRef com ActionIds para focar, selecionar, alternar inclusão/remoção, inspecionar node/segment, acessar handles e invocar ações pertinentes. Navegar por foco **não** modifica automaticamente a seleção. O leitor de tela anuncia nome/tipo, posição, escopo, elegibilidade e estados úteis sem narrar cada frame de drag.

Devem existir caminhos equivalentes para: entrar e sair de Vector Edit; alcançar node sobreposto; invocar Select All/Invert e Marquee/Lasso; alterar tipo; mover nodes por **incrementos configuráveis em unidades do documento**; abrir campo para coordenadas absolutas/relativas; e confirmar/cancelar com Undo atômico quando houver mutação. Incrementos não dependem do zoom do canvas. O mapeamento concreto de teclas e o modelo de focus traversal permanecem na etapa de acessibilidade detalhada.

Ações como Smart Delete e Clean Vector precisam de configuração/feedback textual e via teclado. Preferências de tamanho de targets, foco, densidade de overlays, velocidade de drag e incremento de nudges devem ser controláveis; pressure não muda a semântica da seleção.

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

## 13. Registro de aprovação e próximos assuntos

**Decisões A–F aprovadas em conjunto em 2026-10-08:** A. Marquee de Contenção por padrão, Interseção por ação e Direcional opt-in; B. Lasso contextual com Replace/Add/Subtract; C. bounds discretos automáticos e desativáveis para 2+ nodes; D. lista contextual/ciclo de alvos sobrepostos com alternativas ao Alt; E. Shift alterna seleção e ações de navegação profunda são configuráveis; F. árvore semântica e nudges independentes de zoom. Estas decisões não alteram o modelo autoral e não implicam implementação.

**Aprovado/documentado na etapa seguinte:** handles e suas constraints, criação/recolhimento/desvinculação, tangentes em zero-length, Auto Smooth one-shot, snapping e constraints durante drag, conversões com preview, manipulação numérica e feedback. Veja [Vector Edit — precisão](#/docs/04-ui/vector-edit-precision.md).

**Fechado pelo [ADR-0012](#/docs/00-architecture/adr/0012-interaction-contract.md):** precedência de hit-test (`Handle > Node > Segmento > Fill`, empate por z-order, ciclo reconfigurável); aparência da context bar (4 estados, layout estável, no máximo 3 ações dominantes); paleta e tamanho de âncoras (8×8 px lógicos de captura, 6 px visual, ambos ajustáveis); ícones e atalhos secundários (conjunto primário + editor reatribuível); drag-scrub (opcional, desligado por padrão); focus traversal (ordem visual, Escape devolve o canvas).

[Vector Edit](#/docs/04-ui/vector-edit-interaction.md) · [Smart Path](#/docs/04-ui/smart-path.md) · [Acessibilidade](#/docs/04-ui/accessibility.md) · [ADR-0011](#/docs/00-architecture/adr/0011-hybrid-vector-edit.md)
