# Worktree — Refatoração das Ferramentas (2026-09-22)

- **Branch:** `refactor/tools-funcionamento-2026-09-22`
- **Path:** `../petunia-tools-refactor-2026-09-22`
- **Base:** `e90a6c9` (HEAD de `feature/ui-shell-rework` em 2026-09-22, sem levar o dirty state)
- **Data:** 2026-09-22

## Escopo autorizado

- Refatoração da **implementação do funcionamento de cada ferramenta** do software
  (ex.: `crates/petunia_design_shell/src/tools/*`: select, pen, pencil, node,
  shape, text, gradient, knife, measure, picker, etc.).
- **Fora de escopo:** UI/UX global, shell, tokens, layout, painéis.

## Isolamento

- Worktree separada para não interferir com o agente que está em
  `feature/ui-shell-rework` no diretório principal.
- Este arquivo é o **commit teste** de criação da worktree.

## Próximos passos

1. Inventariar `crates/petunia_design_shell/src/tools/` (manager, mod, cada tool).
2. Definir contrato comum de ferramenta (estado, eventos, mutação via Command).
3. Refatorar tool por tool, com testes focados.
