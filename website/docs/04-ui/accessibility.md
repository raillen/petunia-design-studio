# Acessibilidade e clareza cognitiva

Acessibilidade é requisito de arquitetura de UI, não etapa final de polishing.

## Teclado primeiro

Toda ação de ponteiro relevante deve possuir rota de teclado quando semanticamente possível. Tab/focus order previsível; canvas possui comandos alternativos para operações precisas.

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
