# Acessibilidade e clareza cognitiva

Acessibilidade é requisito de arquitetura de UI, não etapa final de polishing.

## Teclado primeiro

Toda ação de ponteiro relevante deve possuir rota de teclado quando semanticamente possível. Tab/focus order previsível; canvas possui comandos alternativos para operações precisas.

## Vector Edit — acesso cognitivo e por teclado

O [modelo híbrido aprovado](#/docs/04-ui/vector-edit-interaction.md) não pode exigir mouse para abrir e sair do modo, escolher nodes ou executar ações geométricas. Enter sobre um Path elegível acessa Vector Edit, e Escape cancela a interação antes de sair do contexto. Os controles e actions precisam de nomes semânticos, alternativas de navegação e foco visível.

O usuário deve conseguir responder, sem inferência visual:

- estou selecionando objetos ou editando geometria?
- qual objeto/grupo está aberto?
- qual operação está ativa (Node, Bend, Cut, Width etc.)?
- que Node/Segment está selecionado?
- a operação pode ser executada neste alvo?
- houve Commit, Cancel ou erro?

O canvas deve expor uma **árvore semântica navegável** de objetos, nodes e ações (sem exigir que o screen reader interprete pixels), com foco separado de hover e de Selection State. Mensagens durante drag não podem inundar a saída de voz; priorizar início, mudança significativa, confirmação/cancelamento e erro.

A troca de Node para Bend mantém seleção quando válida, reduzindo esforço de navegação. Editar texto, nomes e valores numéricos tem prioridade de teclado sobre comandos do canvas.

**Decisão de seleção aprovada (2026-10-08):** a estrutura acessível navega PathObject → ContourId → NodeId → HandleRef. Foco e seleção são independentes: chegar ao node por teclado não o seleciona nem altera geometria automaticamente. ActionIds permitem selecionar/desselecionar, alternar candidatos coincidentes, inverter/selecionar tudo no escopo, operar Marquee/Lasso, alternar tipo de node e mover por incrementos configuráveis em unidades documentais independentes do zoom. UI deve comunicar o escopo e oferecer alternativas a Shift e ao Alt frequentemente reservado pelo sistema operacional. Veja [Seleção, nodes e handles](#/docs/04-ui/selection-nodes-handles.md).

## Screen reader

Componentes Qt/QML precisam expor semântica de acessibilidade:
- role
- accessible name
- state
- value
- description quando necessário.

Canvas gráfico exige camada semântica que exponha seleção, objeto atual e ações relevantes, em vez de tentar narrar pixels.

## TDAH — reduzir competição visual

- apenas um foco primário por panel;
- progressive disclosure;
- não piscar/mover UI sem necessidade;
- estados ativos muito claros;
- comandos recentes/essenciais fáceis de reencontrar;
- salvar workspace simplificado.

## Dislexia — legibilidade

- largura de texto limitada;
- line-height generoso;
- alinhamento à esquerda;
- linguagem direta;
- evitar blocos longos em caixa alta;
- ícone acompanhado de nome quando ambiguidade for possível;
- não depender de cor para transmitir estado.

## Zoom e escala

UI precisa funcionar em scaling alto e zoom de texto. Não fixar heights que cortem labels.

## Contraste

Estados normal/hover/focus/disabled precisam contraste suficiente. Focus ring não pode depender apenas de diferença sutil de cor.

## Movimento

Respeitar reduced motion. Animações de docking/transições são ornamentais e devem poder ser reduzidas/desligadas.

## Erros

Mensagem deve conter:
1. o que aconteceu;
2. o impacto;
3. como corrigir;
4. detalhes técnicos expansíveis.

Evitar códigos sem explicação como mensagem primária.

## Documentação dentro do produto

Tooltips curtos; help contextual para conceitos complexos; termos técnicos consistentes com esta matriz. “Rasterize”, “Expand”, “Bake” e “Flatten” não podem ser tratados como sinônimos.
