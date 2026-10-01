# Registro de execução da implementação

**Escopo: Milestone Required (MVP) e V1 Required.** O usuário autorizou executar o [plano](/pt/developers/implementation-roadmap-2026-09-30). O trabalho partiu da baseline auditada `73447647a34f5b4be209415c36e6965c5f71b9b9`; achados e probes da baseline conservam a evidência histórica. Este registro descreve implementação, não anuncia release. **MVP, V1 e o plano completo continuam inacabados.** Candidatos/pesquisas futuros mantêm seus critérios de admissão.

## Implementação atual

| Entrega | Status | Comportamento implementado e trabalho restante |
| --- | --- | --- |
| M0.1 bugs de correção reproduzidos | Implementado; gate automatizado passou | R-tree na revisão zero; flats com geração própria; identidades boolean com vazio; RDP pela distância ao segmento finito; offsets compostos preservam componentes; borracha real; blending com alpha parcial; segurança UTF-8; opacity do objeto aplicada uma vez. Corpus e cobertura visual continuam mais amplos que essas regressões. |
| M0.2 schema/integridade | Em andamento | Schema 3/paths locais e frames de referência dos modificadores, migração explícita da entrada no frame do pai, edição atômica de caminho/frame, validação iterativa do grafo, métricas/referências válidas, desanexação reversível de texto/bindings, cadeias live validadas e acessos mutáveis restritos. Decoding serde bruto ainda exige validação na fronteira de confiança; limites de recursos e corpus hostil mais amplo continuam pendentes. |
| M0.3 transações/histórico/save | Em andamento | Conflitos falham; staging/replay com erro não publica mudanças; undo/redo com falha preserva entradas; histórico limitado e ponto salvo por conteúdo; pacote com save exclusivo, atômico, sincronizado e ZIP limitado. Faltam revisões/precondições de operações, snapshots COW completos, recursos no pacote, recovery e prova sob crash/falta de disco. |
| M1 arquitetura de cena/render/workers | Pendente; correções isoladas implementadas | Consultas de metadados evitam copiar paths em cache hit; hit even-odd preserva buracos; alpha de armazenamento/upload coincide; bytes de 16 bits são little-endian; trabalho/endereço do brush são verificados; cancelamento é terminal. Faltam cena comum, composição fiel, cache de imagens, tile workers e execução real dos jobs. |
| M2 vetores/texto | Gates de integração pendentes | Pen/pencil/node/knife/builder leem projeções no frame do pai e publicam paths atomicamente; perspectiva/transparência/crop convertem entradas mundiais a frames locais. A migração é testada pelos fluxos existentes; edição mundial em todos os casos de grupo/rotação, tipografia/glyph runs/caret/IME e usabilidade completa continuam pendentes. |
| M3 bitmap/seleção/ajustes | Pendente | Corrigir kernels não disponibiliza PixelLayer/MaskLayer persistentes, ferramentas Photo e pintura sob seleção. Preservação da fonte, ajustes live e round trip de recursos/tiles continuam pendentes. |
| M4 arquivos/Linux/release MVP | Pendente | Toolchain Rust fixado; lints estritos corrigidos; gate de migração executável; PDF testado com Poppler; argumentos e percentis de snapshots do benchmark implementados. Faltam pacote Linux, pressão/IME/clipboard/a11y reais, recovery, fidelidade SVG/PNG e tarefas completas de usuários. |
| V1-A precisão/assets/UX profissional | Pendente | Coluna de ferramentas profissionais, texto/styles/assets/symbol instances, preflight e quotas/interoperabilidade de plugins/MCP continuam pendentes. |
| V1-B ICC/CMYK verdadeiro | Pendente | CMM ICC, dados tipados de tinta/perfil, proof/separações/TAC reais, configuração de monitor e validação independente continuam V1 Required. Nenhuma heurística atual foi promovida a prova ICC. |
| V1-C PDF profissional | Pendente; páginas/preflight corrigidos | Apenas superfícies habilitadas, dimensões reais e origem local da página; modo estrito rejeita aproximações/cor desconhecida; inclusão no export possui comando reversível. Faltam glyph embedding, imagens, cena mundial comum, clips/grupos, ICC OutputIntent, spots/overprint, PDF/X-4 e validação independente de impressão. |
| V1-D release e versões futuras | Pendente | Projetos profissionais, sessões longas, corpus de compatibilidade, hardware/impressão e instalação continuam pendentes. Pesquisas/candidatos futuros não foram declarados entregues silenciosamente. |

Contratos: [ADR-003](/pt/developers/adr/ADR-003-local-modifier-frames) e [ADR-002](/pt/developers/adr/ADR-002-local-path-and-integrity). [Referência gerada de fontes/testes](/implementation/contracts.json) e [evidência dos checks](/implementation/checks.json) são revisáveis junto ao patch.

## Continuação do MVP: frames dos modificadores

Milestone Required: [ADR-003](/pt/developers/adr/ADR-003-local-modifier-frames) acrescenta schema 3 com tamanhos de referência locais explícitos e migração das entradas dos schemas 1/2. Crop/perspectiva/transparência acompanham posicionamento e escala desigual sem reescrever fonte/parâmetros; contour é avaliado na referência. Entradas desativadas também migram. Sampling local/mundial de opacity retorna a esse frame e não aloca/ordena stops por amostra.

Bake prepara e valida candidato antes de publicar, preserva geometria mundial rotacionada e máscaras restantes, sem reaplicar modificadores seguintes. Crop vetorial vazio continua vazio, mantendo a fonte. Bake seletivo de contour após outra geometria falha com motivo para usar bake geométrico completo. Crop mundial rotacionado exige máscara poligonal e continua indisponível, sem substituir por AABB. Preview/export existentes ainda precisam da cena comum e renderização espacial completa da máscara.

O gate completo expôs um bug da régua: IDs de guias em milissegundos colidiam em gestos consecutivos rápidos. O comando de domínio CreateGuide agora aloca IDs únicos, preservados pelo histórico. Fixtures de formas criam retângulos explicitamente em vez de depender de geometria substituta para objetos sem forma. Uma regressão de ponteiro prova que perspectiva rotacionada preserva cantos não arrastados e undo restaura o documento original.

## Situação dos achados

F02/F03/F09/F18/F19/F20 têm resultados corrigidos com regressões focais. F01/F10/F11/F12/F14/F17/F21–F25/F28/F29/F35/F37 têm implementações que tratam defeitos específicos; achado/milestone continua aberto onde o aceite mais amplo não foi atendido. Demais achados continuam pendentes. O release exige os gates do plano original, não apenas handler ou teste verde de dispatch.

Defeito adicional encontrado durante a implementação: formatadores de dados `Prefix`, `Suffix` e `NumberDecimals` não serializavam com serde internamente tagged. Campos nomeados no wire format agora fazem round trip de todas as variantes, preservando unidades/Currency legados. A validação também exigiu excluir um caminho limpando suas referências de texto e bindings reversivelmente.

## Verificação e limites práticos

`cargo xtask verify` passou: 661 testes, sem falhas ou testes ignorados; 57 regressões/checks focais foram adicionados, além de uma nova regressão de ponteiro do shell e dois doctests compile-fail de fronteira da API. A continuação de frames adicionou 15 casos de regressão ao todo. Formatação estrita, Clippy e restrições de dependências passaram. Os checks legíveis por máquina registram comandos exatos, resultados e totais; os testes são headless, incluindo fixtures de interação Freya TestingRunner e parsing PDF independente por Poppler. Documentação mantém paridade EN/pt-BR e verificação estrita de links. O teste debug do compositor agora verifica pixels/culling, substituindo um limite frágil de 50 ms; performance pertence aos benchmarks release controlados.

O benchmark agora respeita `--objects`, `--iterations`, `--warmup` e `--json`. JSON separa construção da fixture, snapshot frio e média/p50/p95/p99 de consultas aquecidas, com plataforma/profile e digests da cena. Exclui pintura/GPU/apresentação e profiling de memória; smoke runs pequenos não comprovam latência de cauda ou FPS. Uma cópia defensiva no staging introduzida neste trabalho levou a fixture de 10.000 objetos a cerca de 40 segundos; primitivas auditadas com staging direto reduziram isso para cerca de 249 ms, com o mesmo digest. Essas execuções não controladas diagnosticam a regressão interna, não garantem performance do produto.

O linker esgotou os 32 GiB de disco após builds repetidos. Foram removidos apenas artefatos gerados de incremental/app; fontes, evidência de auditoria e arte foram preservadas. Checks que falharam antes das correções não são evidência final de aceite.

Este ambiente não fornece validação de desktop gráfico/tablet/impressão física. Buffers de paths são compartilhados; cópias de contêineres/imagens e estimativa de payload do histórico são provisórias para projetos grandes. Prumo CLI está ausente: este registro acompanha escopo/evidência sem alterar metas Prumo locked ou declarar execução da CLI. Licenças de perfis ICC, fontes/RIP, Wayland/X11, acessibilidade e tarefas reais exigem fixtures/equipamentos declarados e continuam abertas.

Próxima dependência: concluir contratos de recursos/mutação/limites de M0; depois implementar cena comum e composição fiel em preview/export antes de habilitar ferramentas restantes ou declarar o MVP completo.
