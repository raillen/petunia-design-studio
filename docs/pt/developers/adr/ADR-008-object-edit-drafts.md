# ADR-008: Rascunhos estáveis de edição de objetos

- **Status:** Accepted contract; bounded checks only / contrato aceito com checks delimitados.
- **Date:** 2026-10-01
- **Scope:** Milestone Required (MVP)


**Extensão atual:** [ADR-009](/pt/developers/adr/ADR-009-persistent-raster-and-native-workflows) substitui pendências de esquema/recursos, estilo/limites de texto, arquivos/área de transferência nativos e rejeição geral de imagens ICC RGB. Este registro da etapa original é histórico; o código atual não foi validado.

## Contexto

Renomear alternava `*`; texto usava um rascunho compartilhado e reinicializado durante renderização, enquanto Aplicar recriava tipografia padrão e removia texto em caminho. Redimensionar zerava a rotação e não oferecia posicionamento preciso.

## Decisão

O desktop possui um diálogo temporário explícito, identificado pela sessão e ObjectId. Apenas a propriedade editada é comparada ao valor inicial antes de um comando pela ponte. Cancelamento e rascunhos inválidos/conflitantes não publicam. Conteúdo altera uma cópia do descritor; posicionamento declara pontos/graus e dimensões positivas. O modal bloqueia atalhos de canvas. Valores inalterados não geram histórico. Controles legados e edição direta de texto preservam seus limites de escopo.

## Consequências

O buffer global de texto foi removido. O conteúdo usa um modal multilinha no canvas e inspetor. Erros de conflito preservam o rascunho e exigem reabrir com os dados atuais. Multisseleção, controles completos de propriedades, IME e acessibilidade externa permanecem abertos. Sem alteração de schema ou contrato de renderização.

[Rascunhos estáveis de edição de objetos](/pt/developers/uiux-object-edits); [evidence/evidência](/implementation/uiux-object-edits.json).
