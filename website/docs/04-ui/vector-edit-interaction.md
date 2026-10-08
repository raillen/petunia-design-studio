# Vector Edit — interação híbrida contextual

**Estado:** modelo de interação Select + Vector Edit **aprovado em 2026-10-08**. Este documento detalha seu contrato funcional. Atalhos secundários, apresentação definitiva, densidade da toolbar, cores, tamanho dos alvos e algumas escolhas de gestos continuam reservados à revisão conjunta de GUI/UX.

## Objetivo

**Editar o conteúdo vetorial sem alternar desnecessariamente entre ferramentas, sem perder previsibilidade ou confundir seleção com mutação.**

O Petunia dispõe de:

1. **Select:** seleciona e transforma objetos e navega na hierarquia.
2. **Vector Edit:** contexto de edição de geometria, acessado a partir de paths, que oferece Node, Bend, Cut, Smooth, Width e ações relacionadas.
3. **Pen:** ferramenta de criação/continuação de paths, que também pode ser ativada dentro do contexto de edição.
4. **Smart Region:** possui contexto de regiões próprio; pode ser iniciado a partir de Vector Edit mas suas regiões calculadas não viram nodes persistentes.

Nenhuma ferramenta passa a executar uma operação destrutiva só porque um segmento entrou em hover.

### Distinção fundamental

~~~text
Select / Scene Navigation
    │ double-click path, Enter on eligible selection, explicit Node
    ▼
Vector Edit Context
    ├── Node (operação padrão)
    ├── Bend
    ├── Pen / Continue
    ├── Cut / Scissors
    ├── Smooth / Simplify
    ├── Width
    └── ações contextuais: Clean, Region Paint, Shape Builder...
    │ Escape / Exit editing
    ▼
Select / previous context
~~~

**Vector Edit é um contexto de sessão**, não um SceneItem e nem um formato de arquivo. **Node, Bend e outras são operações/controladores**, não novos documentos ou modos de persistência.

## Referências — o que adaptar

- Illustrator: Select e Direct Selection distintos, hierarquia/isolation, precisão e handles.
- Figma Draw: ações de edição vetorial descobertas num contexto comum e manipulação direta.
- Inkscape: edição de segmentos, paths e operações de nodes com feedback preciso.
- Affinity Designer: controle contextual compacto e edição direta no canvas.

Evitar tanto menus fragmentados em dezenas de ferramentas quanto um único cursor que faça várias coisas imprevisíveis.

## Estado de sessão

Direção conceitual:

~~~rust
pub enum EditContext {
    Scene,
    Group { group: ObjectId },
    Vector {
        targets: Vec<ObjectId>,
        active_operation: VectorOperation,
    },
    // Text, Shape e Symbol possuem contextos próprios.
}

pub enum VectorOperation {
    Node,
    Bend,
    Pen,
    Cut,
    Smooth,
    Width,
}

pub struct ToolSession {
    pub navigation_tool: NavigationTool,
    pub context_stack: Vec<EditContext>,
    pub captured_interaction: Option<TransientInteraction>,
}
~~~

Esses tipos **não são proposta de introduzir mais uma crate** nem substituir a tipagem já existente sem migração. A arquitetura visa separar: ferramenta para navegação, contexto hierárquico, operação vetorial ativa, seleção e interação capturada.

A seleção autoral não existe: `SelectionState` guarda `ObjectId`, e a sub-selection guarda `NodeId` / segmento localizável por IDs/revisão. Tudo pertence a Session State.

## Entrada no contexto

| Origem | Evento | Resultado |
|---|---|---|
| Select | Duplo clique em um Path elegível | Empilha contexto Vector Edit para o Path |
| Select | Enter com um Path elegível selecionado | Entra em Vector Edit com operação Node |
| Select | Ativar Node com Path selecionado | Entra em Vector Edit |
| Select | Ativar Node sem seleção | Arma Node; próximo Path elegível pode ser adotado pelo contexto |
| Select | Duplo clique em Group | Entra em Group Isolation, **não** em Vector Edit |
| Select | Duplo clique em Text | Entra em Text Editing, respeitando foco de digitação |
| Select | Duplo clique em ParametricShape | Abre contexto de Shape Editing, sem Convert to Curves |
| Select | Duplo clique em SymbolInstance | Contexto de instância/override, não edita definição automaticamente |
| Select | Duplo clique em Image/PixelLayer | Contexto próprio de Image/Raster; não simula Path |
| Vector Edit | Duplo clique em outro Path elegível | Troca/alarga alvo somente mediante intenção clara; não navega arbitrariamente para fora do contexto |

**Multi-selection:** Enter entra em Vector Edit multi-path somente se todos os objetos selecionados forem paths elegíveis e pertencentes a um escopo de edição compatível. Seleção mista não converte nem filtra silenciosamente: exibir ação explícita `Edit selected paths` que informa o subconjunto e o que ficará fora. Operações que exigem um único path ficam desabilitadas com motivo.

**Eligibility:** hidden, locked, read-only, unsupported Symbol override ou alvo fora do contexto ativo não pode ser editado. Objetos visíveis porém bloqueados podem aparecer no canvas, mas não entram no conjunto mutável. Não abrir editor com alvo que não resolve.

### Entrada por duplo clique sem mutação incidental

Um primeiro click pode mudar Selection State; não produz movimentação. O reconhecimento de duplo clique não dispara duas operações autorais. Se a primeira ação iniciar drag, a operação de duplo clique não deve interromper a captura.

## Context Stack e navegação

Um breadcrumb semântico pode mostrar:

~~~text
Document  ›  Page 1  ›  Group A  ›  Path 03
~~~

O breadcrumb é uma projeção do contexto, não um novo objeto persistente. Pode abrir nível pai e ajuda a recuperar orientação quando existem groups aninhados.

### Regras de Escape

Resolver na seguinte ordem:

1. Popover/diálogo que capturou foco trata Escape segundo seu contrato (sem repassar automaticamente ao canvas).
2. Se há interação transitória capturada (drag, Pen segment, Bend, selection marquee), **cancelar esta interação** sem Commit.
3. Se há operação contextual subordinada com estado interno ativo, voltar à operação Node, preservando seleção coerente.
4. Se Vector Edit está ocioso, sair para contexto anterior, preservando Object Selection; limpar/guardar sub-selection apenas em Session State.
5. Se Group Isolation está ocioso, subir ao grupo pai.
6. Em Select raiz sem contexto aberto, Escape pode limpar a seleção, quando não houver editor de texto/campo com foco.

Sair do modo por toolbar ou breadcrumb, sem commit pendente, usa a mesma lógica do contexto. Nunca descartar silenciosamente edição já confirmada; nunca commit no Escape.

### Enter e foco

- Enter confirma Commit de interação ativa somente quando o controlador da operação definir esse significado, por exemplo Pen concluir um contour.
- Enter no Select com Path selecionado entra em Vector Edit.
- Enter dentro de campo de propriedade ou editor de texto pertence primeiro ao controle focado; não ativa Vector Edit por acidente.
- Node navigation por teclado deve ter caminho viável sem mouse, mas tab/order, setas e modificadores serão refinados na etapa de acessibilidade.
- O canvas não intercepta teclas reservadas ao gerenciador de janelas ou entrada de texto.

A próxima camada de especificação está em [Seleção, nodes e handles](#/docs/04-ui/selection-nodes-handles.md). Ela detalha hit-test, multisseleção, visibilidade de handles, constraints e a proposta de context bar; tipos e indicadores dos nodes, handles progressivos e precisão estão aprovados, enquanto marquee, transformação de multisseleção, gestos e precedência de alvos sobrepostos seguem em revisão conjunta.

## Seleção e sub-selection

**Select:** click escolhe objeto elegível mais alto no hit-test; multi-select pode adicionar/remover; click vazio limpa seleção por padrão; marquee iniciado no vazio. Select Behind / Cycle Overlapping deve ter ação explícita, sem exigir Alt+click (frequentemente reservado pelo WM no Linux).

**Vector Edit:** click em Node seleciona o NodeId; click em segmento identifica o segmento como alvo local; clique vazio limpa a sub-selection, **sem sair do contexto**. Marquee/lasso atuam sobre nodes dos paths elegíveis, sem selecionar objetos fora do contexto por acidente.

**Não há modo de alteração oculta:** arrastar handle move handle; arrastar Node move node; arrastar segmento somente inicia Bend quando Bend está ativo. Na operação Node padrão, arrastar segmento não deforma a curva sem ação/contexto explícito.

### Hit-test e precedência

A ordem leva em conta operation, zoom, target eligibility e hit region:

| Contexto | Prioridade proposta |
|---|---|
| Select | bounding handles explícitos → strokes/fills elegíveis por z-order → fundo para marquee |
| Vector/Node | candidatos node/handle elegíveis → segmento/outline → vazio; a precedência fina de alvos coincidentes **ainda será fechada** com rota de desambiguação |
| Vector/Bend | nodes/handles protegidos → segmento para deformação → vazio |
| Vector/Width | width points/handles → spine para criar width point → vazio |
| Vector/Cut | segment/param position sob cursor → vazio |

Hit-test usa tolerância em **screen-space lógico** convertida pela view transform, com cuidados para zoom, canvas rotation e DPR. Quando alvos competem, mostrar candidato atual e permitir ciclo ou ampliação; não escolher arbitrariamente node adjacente.

### Locked/hidden e múltiplos paths

Paths bloqueados não entram em operações mutadoras. Multi-path editing mantém identity ObjectId, ContourId, NodeId e ordenação autoral. Agrupamento, symbols e masks impõem contexto de scope; operações inválidas são indisponíveis com explicação, não desaparecem silenciosamente.

## Modelo de gestos

| Evento | Select | Vector/Node | Vector/Bend |
|---|---|---|---|
| Hover objeto/path | Outline/hint | Outline da source editável | Segmento deformável em destaque |
| Pointer down em alvo | Select; prepara drag se elegível | Subselect node/segment | Fixa segmento e parâmetro inicial |
| Move antes de threshold | Não muta documento | Não muta documento | Não muta documento |
| Move com capture | Move preview de objeto(s) | Node/handle preview | Bend preview |
| Pointer up | Uma Transaction se moveu | Uma Transaction se mudou path | Uma Transaction se deformou path |
| Escape durante capture | Descarta preview | Descarta preview | Descarta preview |
| Hover vazio | No-op | No-op | No-op |

O threshold para distinguir click de drag é um parâmetro de interação em logical pixels, sujeito a calibração por mouse/touch/stylus; não congelar número arbitrário. Multi-pointer e pan de viewport não podem roubar capture de uma operação já iniciada.

## Ferramentas dentro de Vector Edit

### Node — baseline

Mostra anchors relevantes, handles dos selecionados, escolha Cusp/Smooth/Symmetric, split/join/close, multiselection, edição por teclado. Não criar todos os handles no viewport simultaneamente se atrapalhar leitura; há configuração de visibilidade.

### Bend

Click + drag sobre segmento modifica Bézier por solver geométrico, com ghost de original opcional e error bound. Endpoints não mudam sem escolha explícita; preview e Commit usam mesma matemática. Sem solução estável, feedback de erro, sem mutação.

### Pen

Pen inicia/continua contour elegível, fornece preview de novo segmento e snapping. Confirmar e fechar respeitam estados do controller. Não criar novo Path escondido ao entrar no modo; a escolha Create New/Continue Existing é explícita.

### Cut, Smooth e Width

Ativam uma operação distinta compartilhando o mesmo contexto e sub-selection. Cut produz preview de split; Smooth aplica constraints/tolerance; Width ajusta StrokeStyle profile e não os nodes do Path. Ações só habilitadas quando source suporta o recurso.

### Smart Region

Shape Builder, Region Paint e Weave podem ser iniciados a partir de seleção de paths e abrem um subcontexto próprio para análise de regiões. Enquanto o job de planar subdivision está rodando, mostrar estado de cálculo e manter o canvas utilizável. Voltar mantém a seleção de objetos/paths sem exigir nova seleção.

## Barra contextual, feedback e acessibilidade

A UI deve expor 3 camadas de affordance:

1. **Modo presente:** Select / Vector Edit, objeto/grupo atual, caminho de retorno.
2. **Operação ativa:** Node/Bend/Cut/...; ação de sair/trocar; controles relevantes e estados enabled/disabled com motivo.
3. **Alvo sob pointer e operação em andamento:** outline, node/segment/handle, ghost, snap, error hint.

Não manter todos os controles avançados sempre visíveis; modos básicos claros, resto em menu contextual/overflow e command palette. Ícones com label quando ambíguos; tooltips nomeados e localized; não depender apenas de cor para mostrar seleção.

**Keyboard/screen-reader route:** cada operação essencial precisa ser acessível via ActionId, seleção semântica e comando equivalente quando ponteiro não é possível. Canvas semântico deve anunciar modo, objeto alvo, tipo de node e estado da operação. Texto/inputs e leitor de tela preservam foco; avisos não disparam live-region excessivamente durante o drag.

### Feedback de estado

| Estado | Resposta |
|---|---|
| Hover | visual discreto, sem mudança de Selection |
| Selected | marcador visível distinto de Hover e Active |
| Captured/Dragging | preview + ghost quando útil |
| Disabled/Locked | razão discoverable; sem pointer capture |
| Snapped | guia + tipo de candidato; não só cor |
| Invalid | mensagem contextual corrigível, não toast repetitivo |
| Committed | atualização normal do canvas; toast só se houver informação útil |
| Cancelled | retorna estado visual à revisão base, sem HistoryEntry |

## Operações e fronteiras

~~~text
Qt native events
    ↓ normalized Pointer/Key Actions
UI ToolController + EditorSession context/selection
    ↓ Engine hit test / snap / geometry query
Transient Preview (Session State)
    ↓ confirm
Semantic Command → prepared atomic Transaction
    ↓
petunia-core authoring state + History + Revision
    ↓ snapshot/evaluation → Render Model → Render overlays
~~~

ToolController não recebe `&mut Document`, `QQuickItem` nem implementa geometry algorithm. Qt não monta DocumentOps. Scripts e plugins reutilizam Commands sem simular pointer.

## Erros e alterações externas

Se durante uma captura a revision do documento muda (outro comando/job), verificar expected revision na confirmação. Se base stale, descartar/recalcular mediante regras explícitas; nunca escrever transform baseado em source antiga silenciosamente. Undo/Redo durante captura precisa cancelar ou encerrar a captura antes de navegar histórico.

Se a seleção editável desaparece, é bloqueada ou passa para outro contexto, cancelar interação pendente, preservar SelectionState coerente e mostrar aviso discreto.

## Adoção incremental no código atual

O código presente em `crates/petunia-ui/src/input.rs` possui `ToolKind { Select, Pen, NodeEdit, Rectangle, Ellipse, Zoom }`; `app.rs` cria retângulo fixo 100×100 no pointer down e não implementa capture/context stack. `petunia-engine/src/command.rs` ainda muta Document diretamente, e `snapping.rs` usa threshold escalar fixo. Estes são **gaps verificados** e não representam o contrato final.

Ordem de implementação:

1. Input normalizado (Down/Move/Up/Cancel/Key, device/modifiers, view/world transforms), focus arbitration e pointer capture.
2. EditorSession com ContextStack + SelectionState + TransientEdit.
3. Hit-testing semântico e eligibility; Select verdadeiro com multi/inside/cycle.
4. Node mode, NodeId/ContourId canônicos, seleção por IDs, handles e preview.
5. Engine Command/Transaction atomic + same preview math; implementar Pen/Shape drag corretamente.
6. Bend/Smart Delete/Smooth/Clean integrados às operações de Geometry Engine.
7. Cut/Width e integração com Smart Region conforme fundações estiverem prontas.

## Acceptance tests

| Caso | Resultado obrigatório |
|---|---|
| Double-click Path | Entra Vector Edit sem HistoryEntry |
| Double-click Group/Text/Shape | Abre contexto correto, sem conversão implícita |
| Enter em Path selecionado | Entra Vector Edit; sem Document mutation |
| Enter com foco em input/text | Pertence ao controle focado |
| ESC durante Bend drag | Cancela preview; sem commit |
| ESC depois do drag commit | Volta um contexto, não dá Undo |
| ESC enquanto Pen desenha segmento | Cancela somente segmento transitório |
| Node click → segment drag em Node mode | Não inicia Bend acidental |
| Vector Edit com path locked | No-op com motivo |
| Multi-path com seleção mista | Não descarta objetos silenciosamente |
| Tool switch Node↔Bend | Mantém Object Selection e NodeIds válidos |
| Zoom/DPR/rotated view | Hit-test e drag consistentes em espaço correto |
| stale revision no Up | Não aplica geometria baseada em source obsoleta |
| Group nested + Escape | Desempilha exatamente um contexto por vez |
| Screen reader / keyboard | Permite recuperar modo e selecionar objetos/nodes sem ponteiro |
| Undo/Redo | Uma interação confirmada corresponde a uma transação lógica |

Nenhum desses contratos declara implementação concluída. Testes automatizados e inspeção manual ainda serão necessários.

## Assuntos preservados para discussão

- aparência final de toolbar/context bar/breadcrumb;
- tamanhos dos handles, cores e densidade de overlays;
- atalhos alternativos e conflitos com Linux WM;
- se edição direta de segmento fica disponível em Node após opção explícita;
- presets de interação simplificado/avançado;
- tooltips e progressive disclosure no mobile/tablet quando relevante.

[ADR do modelo híbrido](#/docs/00-architecture/adr/0011-hybrid-vector-edit.md) · [Tools](#/docs/04-ui/tools.md) · [Sessão e input](#/docs/04-ui/session-input.md) · [Acessibilidade](#/docs/04-ui/accessibility.md)
