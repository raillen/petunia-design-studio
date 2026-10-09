# ADR-0012 — Contrato de interação: Tools, Workspace e Acessibilidade

**Status:** Accepted  
**Decisão aprovada:** 2026-10-09  
**Escopo:** camada de interação do Petunia Design Studio (Tools, Workspace, Acessibilidade, acabamento de GUI). **Não** cria feature implementada e **não** reabre [ADR-0011](#/docs/00-architecture/adr/0011-hybrid-vector-edit.md).

## Contexto

A matriz de implementação mantinha Tools, Workspace, Acessibilidade e GUI/UX em **discussão conjunta** porque o restante da arquitetura ainda não estava fechado. Esse trabalho foi concluído: Core, Engine, Render, Render Model e os gates de qualidade existem e são testados. O que falta é decidir o contrato de interação para que a implementação da GUI tenha uma direção única.

O mantenedor delegou em 2026-10-08 a continuidade das recomendações de UX: aprofundar contratos coerentes sem aprovar cada detalhe pequeno, preservar Core/Engine/Render existentes e manter a distinção entre aprovado, especificado e implementado. Este ADR exerce essa delegação nos pontos que permaneciam abertos.

**Já aprovado e não reaberto aqui:** modelo híbrido Select + Vector Edit (ADR-0011); seleção A–F (marquee por contenção, lasso contextual, bounds discretos, ciclo de alvos, Shift, árvore semântica); manipulação Cusp/Smooth/Symmetric; estados da Pen; precedência do Escape; regra de que hover nunca muta documento.

## Decisões

### D1 — Precedência de hit-test em coincidência

Um alvo é eleito por **tipo antes de posição**: `Handle > Node > Segmento > Fill`. Empate dentro do mesmo tipo resolve pela z-order do Scene (topmost primeiro), que é a ordem que o Render já usa.

Para coincidência perfeita entre candidatos de tipos diferentes, o ciclo de alvos já aprovado (decisão D de A–F) é o caminho, com tecla **reconfigurável** — nunca dependente de Alt sozinho.

**Consequência:** a ordem de hit-test passa a ser a mesma da árvore semântica (`PathObject → ContourId → NodeId → HandleRef`), então o que o ponteiro elege é o que o leitor de tela anuncia. Ver [spatial](#/docs/02-engine/spatial.md).

### D2 — Context bar: contrato de conteúdo, layout estável

A barra contextual tem **quatro estados** já descritos em [Seleção, nodes e handles](#/docs/04-ui/selection-nodes-handles.md) §8, que passam a ser o contrato:

| Estado | Conteúdo |
|---|---|
| Select · objeto(s) | posição/tamanho/rotação, align/distribute, group, edit |
| Vector · nenhum node | operação ativa, ações de path, snapping, visibilidade de handles |
| Vector · node(s) | Cusp/Smooth/Symmetric, split/join/close, X/Y, sub-seleção, ações |
| Vector · segmento | Line/Cubic, conversão, comandos da operação ativa |

Regras:

- o layout **só** muda quando o estado de seleção muda — nunca por hover;
- no máximo **3 ações dominantes** por estado; o restante em disclosure explícito;
- controle desabilitado mostra **o motivo**, não apenas estado cinza;
- campo numérico nunca escreve coordenada incompleta no Document.

### D3 — Targets de ponteiro e âncoras

| Item | Decisão |
|---|---|
| Área de captura mínima | 8×8 px lógicos a DPR 1 (área generosa, não só o desenho) |
| Tolerância em unidades de documento | `screen_tolerance_px / view_scale` — invariante já definida em `spatial.md` |
| Tamanho visual do anchor | 6 px lógicos; desaparece abaixo de 4 px para não vir ruído |
| Caminho acessível primário | árvore semântica + nudges; alvo de pixel é caminho secundário |

Todos esses valores são **preferências ajustáveis**, não constantes enterradas no código. O alvo de pixel jamais é a única forma de alcançar um node.

### D4 — Atalhos: conjunto primário + editor completo

**Atalhos primários** (podem ser reatribuídos, nunca fixos em código):

| Ação | Padrão |
|---|---|
| Select / Node / Pen / Brush / Eraser | `V` `N` `P` `B` `E` |
| Desfazer / Refazer | `Ctrl+Z` / `Ctrl+Shift+Z` |
| Selecionar tudo / Grupo / Desagrupar | `Ctrl+A` / `Ctrl+G` / `Ctrl+Shift+G` |
| Mover seleção para trás/frente | `[` `]` |
| Pan / Zoom | `Space+arrastar` / `Ctrl+±` |
| Confirmar / Cancelar | `Enter` / `Escape` |

**Editor de atalhos** é obrigatório no escopo de acessibilidade: lista completa, reatribuição por ação, detecção de conflito **antes** de aplicar, e restauração do padrão. Atalho é dado de preferência, não literal em QML.

Nenhuma ação essencial depende de tecla modificadora única (proibido Alt-only).

### D5 — Tooltip: nome + atalho, sem depender de hover

- tooltip aparece após **400 ms** de hover **ou imediatamente** quando o elemento recebe foco por teclado;
- conteúdo = nome da ação + atalho atual (lido do editor, não de literal);
- a mesma informação está sempre disponível na palette de comandos e no editor de atalhos, que são os caminhos canônicos;
- tooltip nunca é a única forma de descobrir um atalho.

### D6 — Aparência: tokens, forma antes de cor

- cromo do canvas (painéis, barras, réguas, overlays) é derivado de **design tokens** — canvas/foreground/border/accent — com um único accent de interação;
- três temas obrigatórios: **claro, escuro e alto contraste**;
- tipo de node é indicado por **forma**, não só por cor: cusp = quadrado, smooth = círculo, symmetric = círculo duplo;
- nenhum estado é comunicado apenas por cor: foco, seleção, travado e inválido precisam de forma, texto ou padrão adicional;
- movimento respeita `prefers-reduced-motion`.

Tokens vivem no domínio de UI. **Cores de UI não entram no modelo documental** — invariante já vigente em `color.rs`.

### D7 — Smart Delete: duas ações explícitas, sem modificadora

Fica encerrada a pendência de "não congelar tecla modificadora":

| Ação | Comportamento |
|---|---|
| **Preserve Shape** (padrão) | remove o node e reajusta por curve fitting dentro de tolerância declarada; exibe erro máximo |
| **Hard Delete** (explícita) | une os vizinhos diretamente, sem ajuste |

Se o erro do *Preserve Shape* exceder o limite, o preview **avisa** e exige escolher Hard Delete ou Cancel. Nunca ocorre simplificação silenciosa. IDs sobreviventes e o `ContourId` são preservados.

### D8 — Campos numéricos: digitar, teclar, scrub opcional

Ordem de métodos de entrada, todos sempre disponíveis:

1. digitação explícita;
2. `↑`/`↓` com incremento (padrão 1 px, `Shift` = 10 px) em **unidades de documento**, independente de zoom;
3. drag-scrub **opcional e desligado por padrão** — ligado por preferência.

Perda de foco ou Enter confirma; Escape reverte para o valor de origem do arrasto. `Tab` segue a ordem visual da barra (esquerda→direita, cima→baixo). Escape dentro da barra devolve o foco ao canvas.

### D9 — Workspace: layout padrão e regra de sessão

- layout padrão: **barra de ferramentas à esquerda, canvas ao centro, painel contextual à direita**; conjuntos de painel variam por persona, mas o usuário pode acoplar/desacoplar e salvar arranjos;
- todo estado de workspace (painéis, abas, zoom, pan, ferramenta ativa, seleção) é **Session State** — nunca entra no PTND;
- canvas e painéis precisam ser alcançáveis só por teclado, com foco visível e ordem previsível;
- tamanho mínimo de janela utilizável: 1024×640, com reflow abaixo disso.

### D10 — Ponte com Qt

A GUI permanece **Qt/QML via CXX-Qt**, na borda de UI. Nenhum tipo Qt entra em Core, Engine ou Render. Nada neste ADR autoriza frontend web, egui ou outro toolkit.

## Não decidido por este ADR

- **especificação de pixels e QML concreto** da GUI: é entregável separado, sujeito a revisão própria;
- **métricas visuais** exatas de overlay (espessura de linha de seleção, raio de ghost, duração de animação) — definidas no design system de UI, não aqui;
- **branding e ícones finais** — requisição de asset, não decisão de arquitetura;
- **atribuição de teclas por plataforma** ( macOS `Cmd` vs outros `Ctrl`) — implementada pelo editor de atalhos com perfil por plataforma;
- os pontos de engenharia que a matriz lista como "abertos por evidência" (tile size, worker threads, runtime WASM, backend GPU, HDR) — **não** são assunto deste ADR.

## Consequências

- as páginas `04-ui/*` passam de "pendente" para "contrato aprovado, implementação futura";
- o painel de acompanhamento pode fechar a wave U01 no eixo de **decisão**, sem afirmar funcionalidade implementada;
- novos gates de a11y em [qualidade](#/docs/00-architecture/verification.md) §"Vector Edit contextual" passam a ter contrato para serem testados;
- qualquer mudança em D1–D10 exige novo ADR ou supersession explícita.

## Regra de verdade

Este ADR aprova **comportamento-alvo**. Nenhuma frase aqui significa que a funcionalidade existe no código. Implementação e QA de GUI seguem como trabalho futuro, com evidência própria.

## Referências

[ADR-0011 — Select + Vector Edit híbridos](#/docs/00-architecture/adr/0011-hybrid-vector-edit.md)  
[Seleção, nodes e handles](#/docs/04-ui/selection-nodes-handles.md)  
[Vector Edit — interação](#/docs/04-ui/vector-edit-interaction.md)  
[Vector Edit — precisão](#/docs/04-ui/vector-edit-precision.md)  
[Smart Path](#/docs/04-ui/smart-path.md)  
[Acessibilidade e neurodivergência](#/docs/07-agents/accessibility.md)  
[Sessão e Input](#/docs/04-ui/session-input.md)  
[Spatial Engine](#/docs/02-engine/spatial.md)
