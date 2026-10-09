# Acessibilidade, conforto cognitivo e neurodivergência — gates de implementação

**Resumo:** acessibilidade é critério de aceitação, não acabamento ou skin. Usar `accessibility-reviewer` com `ux-architect` e `design-system-engineer`, skills `cognitive-clarity`, `design-psychology`, `keyboard-accessibility`, `focus-management`, `screen-reader`, `contrast`, `motion-accessibility`, `zoom-reflow`. Especificar comportamento **Qt/QML nativo**, não copiar ARIA web literalmente.

## 1. Princípio de design

Pessoa deve editar com previsibilidade e sem exigir memória de atalhos, pointer fino, interpretação de cores ou sequência complexa de gestos. Prioridade: **reconhecimento em vez de memorização**, ações reversíveis, feedback claro, carga cognitiva controlada e confirmação explícita para mudanças destrutivas.

## 2. Critérios por controle da GUI

| Elemento | Contrato obrigatório e exemplo de avaliação |
|---|---|
| Toolbar/menu | labels compreensíveis, tooltips com nome + atalho, seleção/pressed/disabled/hover distintos e anunciáveis |
| Canvas e nodes | target de hit test maior que marcador visível, foco independente da seleção, escolha por teclado/inspector |
| Nodes e handles | estado Cusp/Smooth/Symmetric por shape+label; não só cor; APIs de conversão e numeric input |
| Docking/painéis | restaurar layout, foco previsível ao abrir/fechar, mover painel sem perder editor ativo; sem keyboard trap |
| Trees (Layers/Assets) | selection ≠ focus ≠ expanded; navigation/setsize/level/controls sem scroll horizontal oculto |
| Sliders e number fields | teclado, incrementos configuráveis, edição por número, Apply/Cancel de draft, unidade e range expressos |
| Dialog/modals | foco inicial significativo, Escape correto, Enter sem commit acidental, retorno de foco ao trigger |
| Tooltips e popovers | sem disfarçar controles essenciais, suporte mouse/stylus/teclado, tempo previsível e conteúdo consultável |
| Toast, progress, alerts | mensagem nomeia causa, consequência e ação; status anunciado sem spam; progress cancelable |
| Colors | contraste, modo alto contraste, símbolos, padrões, labels e números para estados de cor |
| Canvas zoom e painéis | texto escalável, density presets, targets compatíveis com DPI, reflow em painéis quando viável |
| Motion | reduced motion, transição discreta, sem animação obrigatória para identificar ação, sem flicker excessivo |

## 3. Fluxos específicos ao Petunia

**Vector Edit:** Zoom/rotation/DPR devem preservar precisão de hit-tests e nodes. Hover não edita; `Node` não se comporta como `Bend`; `Shift+click` e equivalentes ActionId de seleção; Tab/arrow navigation pelos caminhos, contornos, nós e handles. Alt não é dependência exclusiva no Linux.

**Paint:** brush control com feedback numérico de diâmetro, hardness, flow e opacity; inputs de pressão possuem alternativa via slider/campo; layer active target e mask editable target sempre nomeados, `Esc` cancela stroke ainda não confirmado.

**Color:** swatches com números/papel, não só chips; Color Space/Spot status explícito e warnings recuperáveis.

**Photo:** presets, progress de processamento e comparador Before/After, low-memory fallback e cancel, sem exigir ir por quatro painéis para um ajuste comum.

## 4. Requisitos sensíveis a TDAH/dislexia (design universal, nunca estereotipar)

- Reduzir decisões por tela via **progressive disclosure**: 1–3 ações dominantes na barra contextual, advanced settings sob disclosure explícito.
- Nomenclatura consistente, tooltips de verbos, search/command palette, breadcrumb de contexto.
- Ajustes de densidade, tipografia legível e espaçamento confortável; não exigir fonte única como remédio universal.
- Evitar estados que mudam automaticamente sem feedback (auto-apply, auto-delete, auto-switch tool).
- Undo/Redo confiável, previews e cancel: recuperação de erros sem perda de contexto.
- Distratores visuais controláveis: reduzido motion, intensidade de overlays, efeitos selecionáveis e notificações não intrusivas.
- Separar mensagens de erro do usuário de falhas internas; linguagem clara e possibilidade de copiar diagnóstico.
- Testar com pessoas e tarefas reais quando possível; o checklist é base, não substitui pesquisa com usuários neurodivergentes.

## 5. Qualidade e automação

**WCAG 2.2 AA** é referência de objetivos perceptivos/operáveis/compreensíveis; interfaces desktop Qt também dependem de APIs nativas de acessibilidade e testes por plataforma (e.g. AT-SPI em Linux, UIA Windows, NSAccessibility macOS). Não prometer conformidade AA somente porque controles QML usam roles ou screenshots.

**Para cada control novo:** apresentar matrix de `default/hover/focus/active/selected/disabled/error/loading` e testes para teclado/foco/narrativa/accessibility inspection. Confirmar `QAccessible`/semantics disponíveis no toolkit e bridge, sem inventar API QML.

**Neurodivergência:** realizar heuristic audit com `cognitive-clarity` + `design-psychology`; feedback humano quando disponível. Dados de medição podem incluir tempo até ação, miss clicks, error recovery, keyboard-only completion e satisfação, mas nunca inventar resultados.

## 6. Gatilho obrigatório de revisão

Qualquer PR que altere toolbar, tool behaviour, dialogues, layer tree, canvas selection, keybindings, focus, tooltip, toast, animation, typography, contrast ou panel docking exige `accessibility-reviewer` e `ux-architect` como competências de revisão — executadas por agentes se disponíveis ou checklist explícito. Falta de test runner/assistive tech real deve ser documentada, não ocultada.

[Guia canônico UI](#/docs/04-ui/accessibility.md) · [Seleção](#/docs/04-ui/selection-nodes-handles.md) · [Catálogo](#/docs/07-agents/workforce-catalog.md).
