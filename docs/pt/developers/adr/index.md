# Architecture Decision Records

Decisões são documentação de primeira classe. Cada ADR registra contexto, decisão, consequências e status de escopo. O registro histórico completo vive nos cadernos canônicos; os registros abaixo são os que todo contribuidor precisa conhecer primeiro.

| ADR | Título | Status |
| --- | ------ | ------ |
| [ADR-001](/pt/developers/adr/ADR-001-effect-chain) | Edição não destrutiva via EffectChain ordenada tipada | Aceito |

## Como escrever um novo ADR

1. Copie o padrão de cabeçalho do ADR-001 (`Status`, `Data`, `Escopo`, `Contexto`, `Decisão`, `Consequências`).
2. Nomeie `ADR-<NNN>-<titulo-kebab>.md` (próximo número livre).
3. Adicione uma linha a este índice **e** ao seu [espelho en-US](/developers/adr/) na mesma mudança.
4. Linke-o a partir da página cujo comportamento ele governa.

Um ADR sem status de escopo (`V1 Required`, `Milestone Required`, `Post-V1 Candidate`, `Research`, `Open ADR`, `Historical`, `Out of Scope`) é rascunho, não decisão.
