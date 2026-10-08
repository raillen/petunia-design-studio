# Smart Path — editar curvas sem lutar com os handles

**Estado:** modelo híbrido Select/Vector Edit aceito; gestos fundamentais documentados. Atalhos secundários, ícones, estados visuais detalhados e affordances finais sujeitos à revisão conjunta de GUI/UX.

## Referências

Figma Draw: multi-node editing, Shape Builder, melhorias em handles e seleção ([referência](https://www.figma.com/blog/introducing-figma-draw/)). Illustrator: edição de anchor/handles, Simplify; Inkscape: path operations e LPE com atenção à quantidade de nodes ([referência](https://wiki.inkscape.org/wiki/Release_notes/1.3.1)).

A proposta Petunia mantém path canônico **Line/Cubic** e IDs estáveis: a interação pode ser simples sem precisar de Vector Networks como formato autoral.

## Modelo de interação aprovado

Smart Path usa [Select + Vector Edit híbridos](#/docs/04-ui/vector-edit-interaction.md). A entrada por duplo clique/Enter/Node, a hierarquia de contextos, a precedência de Escape e a separação entre hover, seleção e mutação estão definidas no [ADR-0011](#/docs/00-architecture/adr/0011-hybrid-vector-edit.md).

O modo **Node é a operação padrão** em Vector Edit. Bend, Cut, Width e outras operações podem ser ativadas mantendo a sub-selection, sem aplicar alterações por simples hover. Um drag em segmento **não deforma** no modo Node sem ativar Bend ou ação explícita.

## Select e Node — fundação

**Select:** click/lasso/marquee; cycle through overlapping objects; select behind; multi-selection; group/inside navigation; respect layer visibility/lock/selection filter. Bounds e handles são overlays derivados. Duplicar é Command, não efeito colateral do controlador.

**Node:** seleção por NodeId, rectangle/lasso, edição multi-path, cusp/smooth/symmetric, convert Line↔Cubic, join/split/close, handles alinhados, subselect sem forçar troca de objeto autoral. Mover nodes precisa de preview/1 Transaction.

**Pen:** estados Idle/Placing/Dragging/Closing/Continuing, preview de próximo segmento, snapping, curva suave ou canto; Escape cancela o segmento transitório, Enter confirma quando aplicável.

## Direct Bend

**Problema:** ajustar visualmente a curva sem localizar handles minúsculos.

**Proposta de fluxo:**
1. Node/Direct Bend ativo; hover destaca o segmento e ponto mais próximo.
2. Pointer down sobre trecho seleciona intenção de deformação, nunca move um node inesperado.
3. Drag exibe curva nova e ghost da original + desvio máximo opcional.
4. Soltar envia BendSegmentCommand; Escape restaura a curva inicial.
5. Controle contextual opcional: preserve endpoint tangents, symmetric influence, local/falloff, strength, max error.

**Engine:** resolver constrained cubic control-point adjustment perto do parâmetro de contato; distribuir alteração de forma estável, proteger extremos, evitar handles dispararem a infinito. Preview e Commit chamam mesma rotina. Sem solução válida: diagnóstico, não teleport.

**Casos:** segmento muito curto, cusp, curvatura extrema, caminho fechado, múltiplas curvas selecionadas, zoom alto/baixo, objeto transformed.

## Smart Delete Node

**Problema:** apagar um node sem destruir a silhueta.

**Proposta:** ação principal de remoção oferece **Preserve Shape**; alternativa explícita **Direct Join/Hard Delete**. Não congelar tecla modificadora nesta página.

**Engine:** dado A-B-C, retirar B e ajustar segmento(s) restantes por curve fitting dentro de tolerância. Expor error bound; se exceder threshold, preview avisa e oferece Hard Delete ou Cancel. Preservar node IDs sobreviventes e contour ID. Não aplicar silently simplification no documento.

**Casos:** cusp nítida; loops; self-intersection; line-line; cubic-cubic; closed contour; endpoints; outros nodes referenciados por Dimension/Brush Width. Referências removidas precisam política explícita.

## Simplify / Smooth / Clean Vector

**Simplify** reduz complexidade por algoritmo/tolerância. **Smooth** altera distribuição de tangentes suavizando sem necessariamente reduzir nodes. **Clean Vector** é um *analisador e executor de correções*, não um botão de simplificação indiscriminada.

Fluxo Clean Vector:
1. analisar source selecionada, sem mutação;
2. listar duplicate/near nodes, zero-length segments, collinear anchors, minuscule loops, gaps, suspicious self-intersections;
3. classificar seguro/risco visual/topológico;
4. mostrar antes/depois, node count, max deviation, possível mudança topológica;
5. escolher subconjunto de correções;
6. Apply = 1 Transaction; reanalisar após operação para medir idempotência.

**Importante:** operações de topologia e aproximação não são a mesma coisa. Não fundir pontos simplesmente porque um hit-test os considera próximos.

## Select Same/Similar

Consultas sem mudar arte. Filtros:
- fill/stroke/width/dash/blend/opacity;
- font family/style/size e TextStyle;
- effect/style/symbol/type/resource;
- color exact/ΔE perceptual, tolerância de espessura;
- escopo selection/page/artboard/document, hidden/locked options.

Resultados podem ser substituição, adição ou subtração de Selection State. Prévia opcional informa contagem e escopo. Undo de documento não é criado por selecionar.

## Feedback e QA

Hover distinto de seleção; handles legíveis com zoom; tooltip por nome + atalho *após decisão conjunta*. Para Clean mostrar relatório antes de aplicar. Testar jitter de input, snap competitivo, teclado, caneta, 10k nodes, undo/redo, cancel, closed paths e localização de referência.

[Operações Engine](#/docs/02-engine/creative-operations.md) · [Core Path](#/docs/01-core/path.md)