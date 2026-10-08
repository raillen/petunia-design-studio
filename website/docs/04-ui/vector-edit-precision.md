# Vector Edit — handles, continuidade, snapping e precisão

**Estado:** decisões de UX aprovadas por delegação em 2026-10-08. Especificação-alvo; **não implementado nem testado**. Interfaces definitivas, cores e atalhos remapeáveis são detalhes de design system e não autorizam alteração silenciosa da semântica.

**Leia antes:** [Select, Nodes & Handles](#/docs/04-ui/selection-nodes-handles.md) · [ADR-0011](#/docs/00-architecture/adr/0011-hybrid-vector-edit.md) · [Path Core](#/docs/01-core/path.md) · [Spatial + Snapping](#/docs/02-engine/spatial.md).

## 1. Objetivo e fronteiras

Editar uma cubic Bézier deve ser tão direto quanto arrastar sua forma, com a precisão dos handles quando necessário. A operação Node não deforma segmentos pelo simples drag: Bend permanece explícito. Hover, foco, seleção, captura e prévia têm estados diferentes.

- **Core:** `VectorPath`, `NodeId`, `ContourId`, `NodeKind { Cusp, Smooth, Symmetric }`, `handle_in/out: Option<Point>`, `SegmentKind { Line, Cubic }`. Não criar quarto NodeKind.
- **Engine/Geometry:** resolver tangentes, constraints, conversões e validações numéricas, inclusive os estados degenerados; o mesmo solver implementa preview e Commit.
- **Engine/Spatial:** snap providers, constraints, distance em logical screen pixels e histerese.
- **UI/Session:** operação ativa, sub-selection/foco, candidato de hit-test, captura de pointer, ActionIds, campos numéricos e preferências por usuário.
- **Render overlays:** marcadores de node/handle, linhas auxiliares, ghost da forma anterior, snap guides e erro contextual; nenhuma overlay é geometria autoral.

## 2. Manipulação de handles — decisão aprovada

| NodeKind | Ao mover um handle | Efeito no handle oposto |
|---|---|---|
| Cusp | move em direção e comprimento independentes | nenhuma alteração |
| Smooth | altera direção e comprimento do handle manipulado | mantém orientação oposta colinear e preserva comprimento anterior, quando definido e geometricamente válido |
| Symmetric | altera direção e comprimento | espelha direção e módulo conforme o handle manipulado |

Para um anchor P com handle de entrada I e saída O, Smooth impõe tangentes colineares com orientação correta de fluxo; Symmetric acrescenta igualdade de módulos. Não tratar 'ângulo igual' como handles no mesmo lado do anchor. Não transformar G1 em obrigação indevida de C1 para segmentos de parametrização/escala diferentes.

**Gestos:**
1. Hover destaca um handle alcançável e indica seu nome sem modificar a geometria.
2. Down captura `NodeId`, lado (`in/out`), `DocumentRevision` e espaço de coordenadas.
3. Drag atualiza somente preview; mostrar curva resultante e handle oposto afetado quando houver constraint.
4. Up confirma uma Transaction lógica quando houver delta autoral finito; Escape cancela sem HistoryEntry.
5. Drag de handle com múltiplos nodes selecionados ajusta **somente aquele handle** por padrão; ação explícita `Transform Selected Handles` pode mover vários de forma coerente.
6. Alteração de `NodeKind` e tangentes em vários nodes aplica-se de modo determinístico, exibindo contagem e eventual impossibilidade por node; nunca aplicar parcialmente sem escolha expressa.

## 3. Criar, recolher, restaurar e desvincular

**Extract Handle:** comando contextual acessível em node selecionado ou lado de segmento. Se ausente, inicializar direção a partir da tangente existente no segmento ou vetor entre neighbors; escolher comprimento inicial limitado e estável no espaço autoral, não calcular comprimento a partir de pixels/zoom. Para um endpoint, somente o lado que pertence a segmento incidente pode ser criado. Se a direção é indefinida (coincidentes/zero-length), abrir ajuste manual/numérico e mostrar aviso.

**Collapse Handle:** ação explícita que retrai o controle ao anchor e representa adequadamente o estado `None` quando esse for o contrato do Core. Arrastar perto do anchor permite preview visual de zero, mas não apaga o handle por mero threshold de UI. A equivalência geométrica entre `None` e handle no anchor não implica equivalência na intenção de edição; conservar a distinção até a ação de confirmação indicada.

**Unlink Tangents:** converte Smooth/Symmetric → Cusp preservando exatamente posições dos handles; ajuste independente do lado escolhido ocorre somente depois. Atalho não pode depender exclusivamente de Alt (Linux WM).

**Relink Tangents:** propõe Smooth ou Symmetric, com preview. Um lado pode ser adotado como referência explícita (handle ativo) e o outro resolvido para alinhamento; quando não há referência, usar escolha determinística documentada e exibida. Nunca alinhar por ordem acidental do Vec/HashMap.

**Reset/Restore Handle:** restauração para configuração válida sugerida pelo Engine não recupera dados inexistentes implicitamente. `Undo` restaura os valores originais da Transaction anterior.

**Path Line vs Cubic:** `SegmentKind::Line` mantém reta independentemente de handles armazenados. Extract/drag que alteraria a forma deve solicitar conversão `Line → Cubic` com preview explícito; nunca usar handle invisível como mutação latente imprevisível.

## 4. Conversão Cusp / Smooth / Symmetric

- **Cusp:** converter somente NodeKind sem alterar os handles; transformação exata da forma.
- **Smooth:** manter anchor/identidade, alinhar tangentes por solver, preferindo manter o handle explicitamente ativo e preservar o comprimento do oposto; se houver só um handle válido, não inventar um segmento inexistente. Avisar quando não houver tangente matematicamente definida.
- **Symmetric:** além do alinhamento, ajustar módulo conforme referência explícita ou política estável selecionável; mostrar antes/depois se alterar curva.
- **Preview de conversão:** quando a forma mudar, mostrar original em ghost e geometria proposta; painel informa `Nodes: N`, `Mudança máxima estimada` quando calculável; não prometer erro visual exato sem cálculo adequado.
- **Multi-conversão:** uma transação atômica; seleção mista inválida indica quantos nodes/paths não são elegíveis e exige confirmação explícita para alterar apenas subconjunto.
- **Fidelidade:** opções `Preserve current shape` quando a restrição for compatível e `Adjust tangents` quando precisar corrigir curva; não declarar que alinhamento impossível pode preservar exatamente a forma.

## 5. Auto Smooth — operação assistida, não quarto NodeKind

`Auto Smooth` é um ActionId/Engine command que calcula tangentes a partir de nodes adjacentes e propõe novas posições dos handles. Após o Commit, os nodes resultantes têm `NodeKind::Smooth` (ou permanecem inalterados, conforme diagnóstico), sem persistir dependência reativa do movimento futuro dos vizinhos.

- Selecionar nodes → Auto Smooth → preview da forma e dos anchors afetados → Apply/Cancel.
- Opções: `Strength`, `Preserve corners`, `Preserve endpoints`, `Maximum deviation`; defaults conservadores e adaptados às unidades.
- Casos com Cusp intencional, extremidades, zero-length, self-intersection e degenerados devem ser tratados separadamente; valores impossíveis geram diagnóstico, não NaN.
- Se no futuro for desejado Auto node que acompanha neighbors continuamente, abrir ADR específico por alterar intenção autoral. Não simular essa persistência com dados efêmeros.

## 6. Snapping de nodes/handles — decisão aprovada

O usuário vê **um comando Snap** com categorias avançadas expansíveis: Nodes, Path Geometry, Guides/Grid, Alignment/Bounds, Intersections e **Handle/Tangent Alignment**; nenhuma categoria inventa um motor novo. Respectivos providers retornam `SnapCandidate` do [Spatial Engine](#/docs/02-engine/spatial.md).

- **Constrain first:** projetar gesto nos graus de liberdade válidos (eixo/ângulo/restrição tangencial) **antes** de gerar e aplicar snap; snap incompatível é ignorado.
- **Histerese:** conservar candidato capturado até ultrapassar limiar de release; UI não precisa mostrar os valores internos por padrão.
- **Candidate priority:** escolha estável por elegibilidade, operação, scope, priority semântica, latch e distância em viewport logical pixels; ordem de HashMap não desempata.
- **Avoid self-snap:** excluir a geometria da própria subseleção em deslocamento salvo `Snap Internally` explicitamente ativado.
- **Multi-node:** mover o conjunto por um delta comum baseado em âncora/reference node ou bounds, preservando as distâncias relativas; não snappar cada node isoladamente e distorcer a forma.
- **Handles:** Snap de ângulo/direção utiliza orientação/tangentes, não apenas as coordenadas da extremidade do handle; preservar Cusp/Smooth/Symmetric.
- **Temporary disable:** ActionId configurável disponível durante drag e também controle de interface, sem depender exclusivamente de Alt.
- **Visual:** etiqueta acessível curta (ex.: `Snap: node`, `Guide X`, `Angle 45°`), destaque do candidato e linha-guia, nunca apenas cor. Avisos de conflito entram no status contextual, não em toasts repetitivos.

## 7. Precisão numérica e teclado

O inspector/context bar pode editar `Anchor X/Y`, `Handle In X/Y`, `Handle Out X/Y`, comprimento, ângulo, delta X/Y e coordenadas locais/globais. Mostrar sempre unidade e referencial (Path-local, Page ou Document). Quando 2+ nodes têm valores diferentes, exibir `Mixed`, não um número falso. Oferecer modo **Apply delta to all** explicitamente.

**Campos:**
- Digitação cria draft temporário; `Enter` confirma após parsing/validação; `Escape` desfaz apenas o draft daquele controle; blur segue política previsível do form.
- Aceitar sinal, decimais e unidades válidas localizadas. `+5 mm` relativo deve ser apresentado como delta, não confundido com coordenada absoluta; não avaliar expressão arbitrária insegura.
- Escrub/drag em rótulos pode ajustar valores com sensibilidade modificável e fine-control; nunca atualizar documento a cada amostra — apenas preview até finalizar.
- `Arrow Nudge` movimenta em incremento configurável em unidades documentais, independente do zoom. Repetição de tecla pode coalescer um gesto lógico de Undo, com fronteiras claras na perda de foco/release; nunca condensar gestos desconexos.
- Rotação/transformações/escala e DPR não alteram a semântica numérica. Coordenadas world↔local são convertidas com verificação de matriz inversível.
- Toda operação essencial tem ActionId equivalente: focar handle, movê-lo numericamente, alternar tipo, gerar/recolher handle, ativar Snap e cancelar preview.

## 8. Estados visuais e context bar

O layout **não muda no hover**. As áreas fixas: (1) `Select / Vector Edit` e breadcrumb; (2) operação `Node | Bend | Pen | Cut | Smooth | Width`; (3) seleção e tipo de node; (4) posição/tangentes; (5) Snap; (6) overflow para operações especializadas. Disclosure progressivo, sem esconder a operação ativa.

- Normal: forma geométrica tipada, contraste suficiente.
- Hover: halo curto e rótulo opcional, sem seleção.
- Selected: preenchimento/outline reforçado, handles pertinentes visíveis.
- Focus: anel distinto de Selected, preservando leitores de tela.
- Captured: marcador de controle ativo, ghost opcional, delta e snap quando úteis.
- Disabled/Locked: feedback + motivo; nenhuma captura ou edição.
- Invalid/stale: motivo legível e rota para corrigir; nenhum Commit silencioso.

Ícones ambíguos recebem nome visível ou tooltip explícito. Sem animações obrigatórias; reduced motion e high contrast respeitados. Toolbar deve suportar scaling sem labels cortadas. `Shift` para alternar seleção é semântico somente em gesto de seleção; durante drag capturado, ação de constranger ângulo pode usar input configurável sem sobrescrever a sub-selection.

## 9. Qualidade/aceitação

Testes mínimos:
1. Cusp move somente o handle escolhido; Smooth mantém alinhamento e comprimento oposto; Symmetric espelha módulo e orientação.
2. Endpoint não cria handle em lado inexistente; Line não vira Cubic por um ajuste invisível.
3. Handle coincidente com node tem rota de desambiguação de alvo.
4. Drag de handle com múltiplos nodes selecionados não move anchors involuntariamente.
5. Cusp conversion preserva geometria; Smooth/Symmetric informam mudanças por preview.
6. Auto Smooth é operação *one-shot*; editar neighbor depois não dispara recálculo oculto.
7. Snapping obedece constraints, latch e desativação temporária; multi-node mantém deslocamento relativo.
8. Nudge em mm permanece igual em zoom/rotação/DPR diferentes; campos não materializam estados de digitação parcial.
9. Cancel, stale revision, Undo/Redo e erro são atômicos; `NodeId` estável.
10. Keyboard/screen reader conseguem chegar a ambos os handles e executar ações sem mouse.

## 10. Referências funcionais externas

- [Figma — Vector edit / handle mirroring](https://help.figma.com/hc/en-us/articles/360039957634-Edit-vector-layers): sem espelhamento, espelhar ângulo, espelhar ângulo e comprimento.
- [Illustrator — Convert anchor points](https://helpx.adobe.com/illustrator/desktop/draw-shapes-and-paths/modify-paths/convert-anchor-points-on-a-path.html): converter anchors e controlar tangentes.
- [Affinity Designer — Node Tool](https://s3-eu-west-1.amazonaws.com/affinity-docs/help/designer/en-US.lproj/pages/Tools/tools_node.html): node, curve, handle snapping e context toolbar.
- [Inkscape — Auto smooth nodes](https://wiki.inkscape.org/wiki/ReleaseNotes047): referência para distinção entre resultado one-shot e comportamento automático persistente.

**Próxima leitura:** [Pen e criação de paths](#/docs/04-ui/pen-path-creation.md) · [Operações Smart Path](#/docs/04-ui/smart-path-operations.md).
