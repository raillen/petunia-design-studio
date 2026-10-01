# ADR-007: Adaptador desktop de fluxos de arquivos

- **Status:** Contrato aceito; a evidência da implementação é limitada aos checks registrados abaixo.
- **Data:** 2026-10-01
- **Escopo:** Milestone Required (MVP)


**Extensão atual:** [ADR-009](/pt/developers/adr/ADR-009-persistent-raster-and-native-workflows) substitui pendências de esquema/recursos, estilo/limites de texto, arquivos/área de transferência nativos e rejeição geral de imagens ICC RGB. Este registro da etapa original é histórico; o código atual não foi validado.

## Contexto

O desktop executava Novo antes da configuração, deixando uma aba após Cancelar. Tokens de Abrir/Salvar como não tinham UI para destino. O diálogo de exportação omitia o DPI escolhido e oferecia opções não consumidas pelo motor. O fechamento alterado não oferecia salvamento, e índices de abas no momento da resposta não identificam suficientemente o alvo.

## Decisão

O adaptador desktop intercepta o ciclo de arquivos por um router único para menu, paleta e teclado. Prompts de configuração não alteram documentos. Destinos são escolhidos em modal com seletor nativo assíncrono e entrada manual alternativa; somente a confirmação explícita faz dispatch. Pedidos de salvar/fechar vinculam-se a `DocumentSession::identity()`. Salvar usa a Action lane existente e restaura a sessão ativa. Salvar antes de fechar nunca força o fechamento após uma falha; Salvar todos não fecha abas até que cada documento alterado esteja salvo.

Opções de exportação e substituição são resolvidas pelo `ExportRequest` tipado existente antes do dispatch. Escolhas sem suporte são substituídas por explicações de capacidade. Novos textos visíveis pertencem ao catálogo EN/PT. Popups de arquivos suprimem atalhos do canvas enquanto estão abertos. Este adaptador não altera contratos do arquivo/schema nativo, mutação de domínio, ICC ou PDF de produção.

## Consequências

O adaptador acrescenta estado efêmero de prompt/feedback/alvo e dependência direta da versão rfd já usada pelo Freya. I/O continua nos serviços de domínio. Resultados do seletor nativo não publicam documentos. Salvar/exportar permanece síncrono após confirmar; encerramento da janela, foco/IME completo e validação externa de acessibilidade permanecem abertos.

Implementação, Atlas e evidências delimitadas: [fluxos de arquivos no desktop](/pt/developers/uiux-file-workflows) e [referência gerada](/implementation/uiux-file-workflows.json). Esses checks não certificam todo o MVP nem afirmam validação real de portais Linux/tecnologia assistiva.
