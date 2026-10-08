# Pen — criação, continuação e fechamento de paths

**Estado:** especificação de UX aprovada por delegação em 2026-10-08; comportamento-alvo, **não implementado/testado**. Preservar [ADR-0011](#/docs/00-architecture/adr/0011-hybrid-vector-edit.md), Core Line+Cubic e a separação entre Session, Engine e Core.

## Propósito

Criar paths previsíveis sem exigir atalhos obscuros ou entendimento prévio de Bézier. O usuário deve perceber claramente se está criando novo Path, continuando Contour aberto, desenhando uma curva, fechando uma forma ou editando um anchor já existente.

**Princípios:** nenhuma mutação por hover; cada interação possui preview e cancelamento; não converter shape paramétrica em path silenciosamente; não inventar conexões topológicas apenas porque coordenadas coincidem.

## Contexto e entrada

- Pen acessível diretamente na toolbar/ActionId, inclusive fora de Vector Edit; dentro de Vector Edit usa o(s) PathObject(s) elegível(eis).
- Ao entrar sem path apropriado, a context bar informa `New Path`; novo PathObject autoral nasce **apenas** quando uma criação for confirmada, não ao armar a ferramenta.
- Se o path ativo tem um contour aberto e endpoint elegível, oferecer ação explícita `Continue Path`; não anexar automaticamente num contour coincidente pertencente a outro objeto.
- Com vários paths selecionados, `New Path` é padrão seguro; `Continue Selected Path` requer escolher um único endpoint elegível através de target picker/árvore semântica.
- Symbol instances, Text, shapes paramétricas, masks, hidden/locked e objetos read-only são respeitados; editar source versus override é ação explícita.

## Máquina de estados de interação

~~~text
Idle
  ├─ New Path → PlacingFirstNode
  └─ Continue eligible endpoint → Continuing
PlacingFirstNode
  ├─ click → ReadyForNext
  └─ drag → CreatingTangent → ReadyForNext
ReadyForNext
  ├─ move → SegmentPreview
  ├─ click → PlaceCorner → ReadyForNext
  ├─ drag → PlaceSmooth → ReadyForNext
  ├─ click first endpoint (if eligible) → ClosingPreview
  ├─ Enter / Finish action → FinishOpen
  └─ Escape → CancelCurrentPreview/FinishOrDiscard
ClosingPreview
  ├─ confirm → FinishClosed
  └─ cancel → ReadyForNext
~~~

Esses estados são conceituais; controller pode representar composição de substates sem criar crates extras. `PointerDown/Move/Up/Cancel` são eventos normalizados (mouse, touch, stylus) e usam pointer capture.

## Gestos básicos

1. **Click** em vazio elegível posiciona anchor Cusp com comprimento de handles zero/ausente conforme política canônica; o preview entre anchors é `SegmentKind::Line` quando houver reta.
2. **Click+drag** para criar um node suave: o gesto define tangente de saída, e a de entrada quando ambos os lados incidentes são cubics; magnitude proporcional ao gesto em coordenadas documentais, respeitando transform e constraints.
3. **Hover no primeiro endpoint** do mesmo contour aberto mostra `Close Contour` com preview e confirmação explícita; não encerrar em outro node apenas por proximidade de screen-space.
4. **Continue Path** a partir de endpoint existente: mostrar `Continue from start/end`, inclusive orientação; não reordenar NodeIds sem comando adequado. O caminho visual permanece atualizado após zoom/pan.
5. **Criar um Cusp após Smooth** não força continuidade; `Shift` ou outro ActionId de constrain opera enquanto há gesto capturado sem roubar Shift+click de multisseleção.
6. **Curva reta vs cúbica:** uma reta tem `SegmentKind::Line` efetivo; manipulação futura que produza curva deve usar `Line→Cubic` explícito via action/preview, salvo o próprio gesto de criação de cubic.
7. **Duplo clique** não deve automaticamente criar dois nodes; não usar double-click como única forma de finalizar path, pois tecnologias assistivas e caneta podem não reproduzi-lo com precisão.

## Preview e intenção visível

Durante a construção, exibir linha/curva proposta, node atual, direção de tangent, candidato de snap e status textual (`New Path`, `Continue Path`, `Close`, `Finish Open`). Exibir ghost da geometria original caso continue um path existente.

- Movimento de cursor sozinho não cria NodeIds nem HistoryEntry.
- Preview de curva usa a mesma rotina Engine que materializará os segmentos. De Casteljau e entidades canônicas são preservadas.
- O botão/ação `Finish Open` sempre está disponível; `Close` apenas quando topologicamente permitido.
- Sair de Pen quando há construção ativa usa controle `Finish / Discard` se o usuário já confirmou anchors dentro de uma sessão de desenho; nunca descartar trabalho confirmado silenciosamente.

## Escape, Enter, Backspace, Undo

- **Escape enquanto arrasta o próximo node/handle:** cancela somente o gesto transitório e volta ao estado anterior.
- **Escape com preview do próximo segmento:** cancela o preview; o estado da construção permanece recuperável.
- **Escape com Pen ociosa:** aplica regra de ContextStack já aprovada, após verificar se existe path incompleto que demanda Finish/Discard.
- **Enter/Finish:** confirma contour aberto quando faz sentido e o controle de canvas possui foco; num campo numérico, Enter pertence ao campo.
- **Backspace / Remove last point:** operação explícita com confirmação/preview quando remover único ponto afetaria o objeto; não confundir com Delete de objetos de Select.
- **Undo durante construção:** deve desfazer o último passo significativo de criação conforme política de History, não destruir uma construção em progresso inteira inesperadamente. Cancelamento de preview transitório precede Undo.
- **History:** cada gesto completo de adição pode ser uma transação lógica; iniciar novo Path e adicionar seu primeiro anchor na mesma transação. Agrupamento eventual do stroke inteiro como um passo de Undo deve ser opção explícita, não a única forma de recuperar anchors individuais.

## Tangentes durante criação

- **Cusp:** entrada/saída independentes.
- **Smooth:** alinhadas, magnitude livre.
- **Symmetric:** alinhadas, magnitude espelhada.
- `Extract Handle`, `Collapse Handle`, `Unlink/Relink Tangents` e `Auto Smooth` reutilizam a [especificação de precisão](#/docs/04-ui/vector-edit-precision.md), nunca outro solver.
- Ao começar um contour, a extremidade inicial só possui o handle efetivamente incidente. Fechar contour pode tornar o segundo lado válido, mas sua criação não é automática sem preview.

## Snapping e constraints

- Reutilizar os mesmos providers de Node/Handle/Guides/Grid/Intersection/Angles de Spatial; `SnapLatch` fica em Session.
- Uma linha-guia indica alvo e tipo; para múltiplos candidatos coincidentes, abrir picker sob demanda.
- A ferramenta Pen permite `Constrain angle` e `Temporarily disable snapping` por ActionId remapeável, sem assumir Alt como única alternativa.
- Não gerar duplicatas de nodes quando há zero-distance snap: oferecer `Close` ou `Join` quando validável; caso contrário colocar anchor no mesmo ponto é permitido como degenerado, mas usuário recebe feedback claro.
- Hit targets grandes não significam tolerâncias geométricas autorais amplas: seleção em px lógicos, join/close por validação topológica.

## Teclado e números

- `Create New Path`, `Continue Endpoint`, `Place Node at Coordinates`, `Insert Tangent`, `Finish Open`, `Close Contour`, `Cancel Segment`, `Undo Last Point` possuem ActionIds nomeados.
- Inspector numérico de ponto X/Y, ângulo e comprimento de tangent trabalha em unidades do documento, local/World explícitos e incrementos configuráveis independentes de zoom.
- Árvore semântica expõe Path → Contour → Anchors/Handles; qualquer operação essencial pode ser iniciada sem movimento de ponteiro. Modo teclado coloca node por coordenadas e permite continuar de um endpoint conhecido.
- Para iniciantes, ajuda contextual discreta com exemplos de clique=reta / arrastar=curva / fechar ao clicar início, desligável e acionável em Help.

## Casos difíceis

- Path fechado: não permitir continuar diretamente sem `Break/Open Contour` explícito, com preview.
- Multi-contour num mesmo path: escolher target pelo id, não pela proximidade aleatória.
- Revisão muda no meio do gesto: verificar expected revision, cancelar/recalcular com aviso e sem geometry stale.
- Transform singular ou coordenada não finita: bloquear comando, devolver erro recuperável.
- Shapes/masks/Symbols: preservam semântica do objeto de origem; `Convert to Curves` é ação explícita separada.
- Existing Path com apenas um node, cubic degenerado, zoom extremo, DPR alto e viewport rodada: comportamentos previsíveis.
- Pen pressionada em input de texto não captura teclas ou drag.
- Quando o cursor sai da viewport durante drag, pointer capture continua até Up/Cancel legítimo.

## Barra contextual de Pen

**Faixa fixa:** `Pen` + `New/Continue` + tipo de ponto `Corner/Smooth/Symmetric` + `Snap` + `Finish` + `Close` (somente elegível). Campos X/Y e angle/length aparecem ao focar anchor/handle. Avançado: policies de append, segment kind, constraints e progressive disclosure.

Estados disabled sempre mostram motivo; não esconder `Finish` ao entrar em submenu. Labels compreensíveis acompanham ícones ambíguos e screen readers recebem nome, descrição e ação.

## Critérios de aceitação

1. Click+drag e clique simples têm resultado diferente e previsível; threshold usa pixels lógicos.
2. Cursor move/hover não deixa documento dirty nem cria nodes.
3. Fechamento produz `closed=true` sem duplicar primeiro node; `Finish Open` mantém contour aberto.
4. Continuar path não muda IDs preexistentes nem edita objeto incorreto.
5. Novos segmentos respeitam Line+Cubic, sem quad persistente.
6. Enter/Escape/Undo obedecem foco e captura; nenhuma perda silenciosa de trabalho confirmado.
7. Snap/constraint operam igual em mouse, stylus e teclado onde aplicável.
8. Revisão obsoleta ou transform inválida não produz commit parcial.
9. High contrast, acessibilidade sem ponteiro e leitores de tela são verificáveis.
10. Metadados de preview não entram em PTND/History.

## Referências

- [Adobe — Convert anchor points](https://helpx.adobe.com/illustrator/desktop/draw-shapes-and-paths/modify-paths/convert-anchor-points-on-a-path.html)
- [Figma — Edit vector layers](https://help.figma.com/hc/en-us/articles/360039957634-Edit-vector-layers)
- [Inkscape — node editing](https://inkscape.org/pt/doc/advanced/tutorial-advanced.pt_BR.html)
- [Core Path](#/docs/01-core/path.md), [Geometry](#/docs/02-engine/geometry.md), [Session/Input](#/docs/04-ui/session-input.md)
