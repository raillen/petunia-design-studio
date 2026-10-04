# Architecture Decision Records

Decisões são documentação de primeira classe. Cada ADR registra contexto, decisão, consequências e status de escopo. O registro histórico completo vive nos cadernos canônicos; os registros abaixo são os que todo contribuidor precisa conhecer primeiro.

| ADR | Título | Status |
| --- | ------ | ------ |
| [ADR-001](/pt/developers/adr/ADR-001-effect-chain) | Edição não destrutiva via EffectChain ordenada tipada | Aceito |
| [ADR-002](/pt/developers/adr/ADR-002-local-path-and-integrity) | Caminhos locais, schema 2 e publicação atômica | Aceito · Milestone Required |
| [ADR-003](/pt/developers/adr/ADR-003-local-modifier-frames) | Frames locais persistentes, schema 3 e bake que preserva aparência | Aceito · Milestone Required |
| [ADR-004](/pt/developers/adr/ADR-004-render-snapshots-and-workers) | Snapshots vetoriais imutáveis e workers limitados | Contrato aceito · Milestone Required · validação pendente |
| [ADR-005](/pt/developers/adr/ADR-005-immutable-image-assets) | Fontes de imagem imutáveis, decoding limitado e pirâmides compartilhadas | Contrato aceito · Milestone Required · validação pendente |
| [ADR-006](/pt/developers/adr/ADR-006-shaped-text-and-canvas-preview) | Contornos de texto com shaping e apresentação da última prévia solicitada | Contrato aceito · Milestone Required · validação pendente |
| [ADR-007](/pt/developers/adr/ADR-007-desktop-file-workflows) | Adaptador desktop de fluxos de arquivos | Accepted contract · Milestone Required · see bounded UI checks |
| [ADR-008](/pt/developers/adr/ADR-008-object-edit-drafts) | Rascunhos estáveis de edição de objetos | Accepted contract · Milestone Required · bounded UI checks |
| [ADR-009](/pt/developers/adr/ADR-009-persistent-raster-and-native-workflows) | Raster persistente, recursos binários e fluxos nativos | Contrato aceito · Milestone Required · validação pendente |
| [ADR-010](/pt/developers/adr/ADR-010-text-histogram-icc-and-pdf) | Texto, histogramas, ICC e PDF fiel | Contrato aceito · Milestone Required / V1 Required · evidência final separada |

| [ADR-011](/pt/developers/adr/ADR-011-native-cmyk-raster) | Raster CMYK nativo, recursos ICC e intercâmbio preservando tintas | Contrato aceito · V1 Required · evidência final separada |

## Como escrever um novo ADR

1. Copie o padrão de cabeçalho do ADR-001 (`Status`, `Data`, `Escopo`, `Contexto`, `Decisão`, `Consequências`).
2. Nomeie `ADR-<NNN>-<titulo-kebab>.md` (próximo número livre).
3. Adicione uma linha a este índice **e** ao seu [espelho en-US](/developers/adr/) na mesma mudança.
4. Linke-o a partir da página cujo comportamento ele governa.

Um ADR sem status de escopo (`V1 Required`, `Milestone Required`, `Post-V1 Candidate`, `Research`, `Open ADR`, `Historical`, `Out of Scope`) é rascunho, não decisão.
