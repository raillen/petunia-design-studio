# ADR-0011 — Select e Vector Edit híbridos contextuais

**Status:** Accepted  
**Decisão aprovada:** 2026-10-08  
**Escopo:** modelo de interação do Petunia Design Studio. Não altera domínio autoral nem define visual final de GUI.

## Contexto

Select/Node/Pen separados são previsíveis, mas forçam muitas trocas de ferramenta. Um modo totalmente automático reduz trocas, porém aumenta risco de ações inesperadas, especialmente ao arrastar segmentos e cruzar boundaries entre grupos, paths e shapes.

A proposta combina seleção de objetos explícita com um contexto de edição vetorial que compartilha sub-selection e ações relacionadas.

## Decisão

O Petunia usará:

~~~text
Select
  ↓ double-click eligible path / Enter / explicit Node
Vector Edit
  ├── Node
  ├── Bend
  ├── Pen
  ├── Cut
  ├── Smooth
  ├── Width
  └── acesso a Smart Region / outras operações
  ↓ Escape/Exit quando ocioso
Select / parent context
~~~

**Select** seleciona/move objetos; **Vector Edit** edita nodes/handles/segments. **Vector Edit é Session State**, não Document State.

Duplo clique em Group, Text, ParametricShape ou SymbolInstance abre o **contexto semântico daquele item**, não converte automaticamente em paths.

A operação selecionada define o que um drag faz: **Node não executa Bend involuntariamente; hover não é autorização de mutação.**

Escape primeiro cancela captura/preview, depois retorna contexto. A seleção de objeto permanece disponível ao sair, e sub-selection continua efêmera.

Toda operação confirmada passa por Command → Transaction; o documento nunca muda no hover ou em preview.

## Alternativas consideradas

### Todas as operações como ferramentas isoladas

Previsível e familiar, porém cria fragmentação excessiva para edição vetorial contínua e mais complexidade de toolbar.

### Ferramenta única com gestos implícitos

Reduz troca de ferramenta, porém pode deformar geometria sem intenção clara e dificulta previsibilidade/acessibilidade.

### Converter shapes/text/symbols em curves para permitir entrada

Rejeitada: destrói intenção autoral e contradiz o sistema não destrutivo.

## Consequências

- ContextStack/SelectionState/TransientEdits ficam no serviço de UI/session.
- Object Selection e Node sub-selection são distintos.
- Node Tool explícita segue disponível.
- Enter/double-click respeitam foco, elegibilidade e contexto de objeto.
- Group isolation e vector editing compartilham navegação hierárquica, mas não semântica.
- Hit-test e snapping pertencem ao Engine, cursor/overlay/feedback à UI/Render.
- Menus, toolbar e command palette operam por ActionId, sem duplicar funções.
- Pointer capture e cancelamento consistente são pré-requisitos antes das operações Smart Path.
- Testes cobrem double-click, Escape, foco de texto, context nesting, locks, multi-selection e undo.

## Não decidido por este ADR

Posições e agrupamento definitivos na toolbar, ícones, labels abreviadas, atalhos secundários, cores, valores absolutos de hit targets, modo simplificado versus avançado. Esses assuntos serão finalizados na discussão de UX/acessibilidade, sem reabrir o princípio híbrido salvo evidência contrária.

## Referências

[Especificação completa da interação](#/docs/04-ui/vector-edit-interaction.md)  
[Smart Path](#/docs/04-ui/smart-path.md)  
[Sessão e Input](#/docs/04-ui/session-input.md)  
[Ferramentas](#/docs/04-ui/tools.md)
