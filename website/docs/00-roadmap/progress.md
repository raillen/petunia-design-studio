# Acompanhamento da implementação

> **Status: protocolo operacional aprovado em 2026-10-09.** Este módulo acompanha a execução dos contratos de `website/docs/` na branch `petunia-design-rust`. A tabela não substitui decisões de produto nem evidências de teste.

[Abrir a tabela de processos](#/progress)

## Fonte única

`website/progress/tasks.json` é a fonte versionada dos processos, checkpoints, estados, datas e evidências. O site lê esse arquivo diretamente, sem build, servidor de escrita ou status privado em localStorage. Alterar código sem atualizar a tarefa correspondente deixa a entrega incompleta.

Cada tarefa tem ID estável, etapa, descrição do delta, documento principal, leituras complementares, checkpoints de aceitação e estado. Todos os capítulos do caderno estão associados a processos; o catálogo Prumo orienta a seleção de especialistas, sem se tornar feature do editor.

## Três estados

| Estado | Regra |
|---|---|
| `TODO` | Implementação ainda não iniciada; zero checkpoints concluídos. |
| `IN PROGRESS (000%)` | Trabalho iniciado com gates ainda abertos. O percentual é calculado pelos checkpoints concluídos e exibido com três dígitos. |
| `DONE` | Todos os checkpoints do escopo delimitado concluídos, cada um com evidência; gates aplicáveis executados. |

Percentual = `floor(100 × checkpoints concluídos / total de checkpoints)`. Exemplo: um de quatro = `IN PROGRESS (025%)`; trabalho iniciado sem checkpoint concluído = `IN PROGRESS (000%)`. Todos têm peso igual: o número mede critérios atendidos, **não tempo, esforço ou percentual global do produto**. Não arredondar para 100% com critério aberto.

Uma tarefa pode voltar de DONE para IN PROGRESS quando uma regressão invalidar a evidência. Reabrir o checkpoint afetado e explicar a causa. Não criar quarto estado para bloqueios: manter IN PROGRESS e registrar impedimento, próximo passo e gates `blocked` na evidência do capítulo.

Os estados de documentação (`APPROVED`, `SPECIFIED`) continuam valendo nos capítulos canônicos; aqui valem os três estados de execução acima, conforme a [taxonomia obrigatória](#/docs/07-agents/authority-reading.md).

## Atualização obrigatória a cada implementação

1. Antes de implementar, localizar os IDs afetados, confirmar documento, código e critérios. Marcar IN PROGRESS quando iniciar; atualizar `updated` na tarefa e no arquivo.
2. Ao entregar um slice, marcar somente os checkpoints comprovados. Anexar referência à entrada de `evidence` com revisão/ambiente, resumo e documento que registra os comandos realmente executados, resultados e limites. Não usar aprovação de ADR como prova.
3. Se houver mudança de escopo, dividir/adicionar tarefa com ID novo e critérios verificáveis. Preservar IDs existentes; não apagar trabalho entregue nem alterar o denominador apenas para melhorar a porcentagem.
4. Registrar no capítulo canônico os gates `not run / pass / fail / blocked`, a revisão validada, riscos e próximo checkpoint. Gates manuais necessários ainda abertos impedem DONE no processo que os exige.
5. Atualizar status, checkpoints, evidência e datas **no mesmo conjunto de alterações da implementação**. Se nada puder ser concluído, atualizar baseline/impedimento/data mesmo que o percentual permaneça.
6. Executar os validadores abaixo, revisar o diff e incluir IDs/status no handoff. A tabela é mantida pelos contribuidores/agentes; o site calcula a apresentação, não detecta automaticamente se uma feature funciona.

## Formato dos dados

Cada checkpoint contém `title`, `completed` (booleano) e `evidence` (ID da prova ou null). Um checkpoint concluído requer prova existente. `documents` começa pelo `document` principal e aponta apenas para rotas registradas no manifesto. A evidência contém `revision`, `document` e `summary`; detalhes extensos ficam no capítulo vinculado.

Exemplo ilustrativo: ao executar o primeiro de quatro checkpoints, mudar o estado para IN PROGRESS e marcar aquele checkpoint como concluído com prova; a interface apresentará `IN PROGRESS (025%)`. Não editar um percentual separado: ele é derivado.

## Validação

```bash
node --check website/app.js
node --check website/progress/progress.js
node website/scripts/verify-progress.cjs
node --test website/tests/progress.test.cjs
git diff --check
```

O validador verifica branch, baseline, IDs, estados/checkpoints, datas, evidências, existência de documentos e cobertura das rotas Markdown. Isso não comprova os gates Rust do editor.

## Snapshot inicial e limites

Inventário inicial: 97 páginas no manifesto + home/about, baseline `52775a9618a8b376a3f2dad70f048fe17bca47d4`. Os contratos foram agrupados em 13 processos de implementação mais T01, o próprio painel. Não são 13 funcionalidades ausentes: math, units, ids, path, scene, document, color, commands, snapping, render backend, software e sessão de UI já existem em forma básica.

C01 inicia em 075% (math, units e ids comprovados por `cargo test -p petunia-core` 14/14); C02 em 050%; U01 em 033% (sessão mínima, GUI fora do fechamento por decisão). Outros processos começam conservadoramente em TODO até confirmação específica; seu campo baseline registra o estado observado no checkout.

A ordem prioritária segue a [matriz de implementação](#/docs/00-architecture/implementation-matrix.md); G01 (geometry booleana) é a próxima fronteira do Engine após commands/spatial. Expansões adiadas (backend GPU, HDR, mesh gradient avançado, tabelas editoriais) não são dívida obrigatória da baseline.

## Evidência da entrega do painel — T01

Intenção: integrar processos vinculados aos documentos e atualização contínua. Fontes: matriz de implementação, [leitura e autoridade](#/docs/07-agents/authority-reading.md) e inventário por domínio. Gap: o site tinha leitura/navegação, mas não uma tabela versionada com estados.

Changed: módulo `website/progress/`, navegação, dados, protocolo e validadores. Nenhuma compilação Rust é necessária para este slice do site.

Verification — 2026-10-09, Linux:

| Gate executado | Resultado |
|---|---|
| `node --check website/app.js` e `node --check website/progress/progress.js` | pass |
| `node website/scripts/verify-progress.cjs` | pass: 14 processos e 99 documentos cobertos; IDs, três estados, datas e evidências consistentes |
| `node --test website/tests/progress.test.cjs` | pass: 6 testes, zero falhas/ignorados; percentuais, estados, dados inválidos, evidência fora do escopo e filtros |
| `git diff --check` | pass |
| Navegador local `http://0.0.0.0:8080/website/#/progress` | a verificar: carga dos processos, busca sem acentos, filtros combinados, reset, estado vazio, permalink U01, checkpoints e ida/volta ao documento/protocolo |
| Layout e teclado | a verificar: viewport padrão e 390×844; corpo sem overflow horizontal, tabela com scroll próprio |
| Console durante o fluxo | a verificar: zero erros capturados |

Risks/limites: a verificação em navegador está pendente neste slice (servidor local do usuário). O painel registra fatos de execução fornecidos pelos contribuidores, e o validador não comprova que a evidência humana é verdadeira. O site não foi publicado.

Next checkpoint: atualizar os IDs afetados na próxima implementação; G01 continua como próxima fronteira do Engine. T01 pode encerrar somente o escopo do painel acima; isso não fecha os gates de produto.

## Auditoria e correções pré-GUI — 2026-10-10

A auditoria reabriu checkpoints cujo comportamento não correspondia à documentação. As 15 regressões da auditoria e as novas integrações passaram; a evidência atual refere-se ao working tree sobre `6e2be4e1ce400e1b4642acc2de1007d7cd5e67c9`, não aos commits históricos citados acima.

E01 fecha a correção de transactions/history no escopo de pruning e hard budget. C03, C04, G01, E02, E03, R01, U01 e Q01 continuam IN PROGRESS: checkpoints explícitos registram appearance ligada, efeitos, guards legados, snap completo, I/O, lifecycle, cache/pool e QA ainda pendentes. A [matriz](#/docs/00-architecture/implementation-matrix.md) separa entregas executáveis dos contratos restantes; [Verification](#/docs/00-architecture/verification.md) registra os resultados.
