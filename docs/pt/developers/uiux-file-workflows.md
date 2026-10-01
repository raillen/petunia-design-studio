# Fluxos de arquivos no desktop — implementação de UI/UX

Data: 2026-10-01. Escopo: **Milestone Required (MVP)**. Esta é a primeira etapa de implementação da auditoria de UI/UX; o redesenho e o MVP continuam incompletos.

O adaptador desktop encaminha ações de arquivo do menu, da paleta e dos atalhos por um fluxo comum. `file.new` abre a configuração sem criar uma aba; a confirmação cria um documento, valida nome/dimensões/sangria/margens antes da publicação e preserva a sessão anterior se a configuração falhar. Escape, Cancelar e o fundo dispensam o modal.

Abrir, Salvar e Salvar como usam um campo de caminho em modal com seleção nativa de arquivo assíncrona. O seletor apenas preenche o campo; a confirmação executa a ação de domínio. Cancelar preserva os documentos e o estado de alterações. O salvamento usa a identidade local `SessionIdentity`, em vez da aba ativa quando chega a resposta, e restaura a aba ativa tanto no sucesso quanto na falha. Pedidos obsoletos falham sem salvar um documento substituto. O destino mostra a extensão nativa `.PTND`. Substituir outro arquivo exige uma segunda confirmação explícita após normalizar a extensão.

Fechar uma aba alterada oferece Cancelar, Descartar e fechar e Salvar e fechar. A identidade é capturada no clique inicial; o ícone de fechar interrompe a propagação para a aba. Fechar todos com Salvar grava cada documento alterado antes de fechar qualquer aba; uma falha ou cancelamento de destino mantém todas abertas. Gravações anteriores bem-sucedidas continuam salvas. Sair atualmente fecha as sessões de documentos; encerrar a janela/processo do sistema permanece fora desta etapa.

Ctrl/Cmd+O, S, Shift+S e W complementam os atalhos existentes de Novo/Exportar. Popups de arquivos/configuração suprimem o handler de atalhos do canvas durante a digitação. Isso não comprova arbitragem completa de foco em campos do inspector, texto/IME ou navegação da paleta. Popup usa o papel de acessibilidade Dialog do Freya; retenção de foco com leitores de tela e comportamento AT-SPI ainda exigem validação em sessão/hardware reais.

A exportação usa o mesmo `ExportRequest` tipado para DPI, destino normalizado, confirmação de substituição e dispatch. PNG mostra o escopo da prancheta ativa e as dimensões previstas em pixels, envia o DPI escolhido e preserva a transparência. SVG/PDF mostram o escopo do documento; PDF respeita as pranchetas habilitadas para exportação. Erros permanecem visíveis com as opções preenchidas, e gravações/exportações bem-sucedidas informam o destino efetivo em mensagem dispensável na barra de status. Opções sem suporte de fundo branco e criação de documento Display P3/CMYK foram removidas e substituídas por explicações explícitas. A UI identifica o PDF como básico e informa que ICC, CMYK e PDF/X para produção estão indisponíveis. Esta etapa não implementa gestão de cor ou PDF profissional.

Os textos dos fluxos de novo documento, destino, fechamento e exportação têm fonte canônica EN e espelho PT sincronizado em `file_workflow_strings.rs`. O catálogo separado evita alterar o catálogo de ferramentas raster durante a continuação do MVP.

## Verificação e pendências

Consulte as [evidências geradas de fontes e checks](/implementation/uiux-file-workflows.json). A verificação usa uma cópia de fontes e um diretório Cargo separados para não consumir os artefatos da tarefa de renderização. Pré-requisitos de compilação encontrados receberam correções mínimas de tipos: tamanho explícito do array RGBA, conversão Arc/Vec de tiles, bounds opcional de objeto raster e metadados desktop da nova variante PixelFill. Essas correções não comprovam a correção das ferramentas raster.

Os testes focais cobrem Novo/cancelar/confirmar, configuração inválida, salvamento de aba inativa, falhas de gravação, identidades obsoletas, salvamento sequencial antes de fechar, preservação do alvo quando índices mudam, erros visíveis, proteção de atalhos em modal, normalização e dimensões reais do PNG. Checks de interação do chrome e localização são registrados separadamente. Executar os testes nesta conversa lateral não altera a política de validação adiada da tarefa principal do MVP nem certifica todo o workspace em evolução.

Portais nativos/Zenity, Wayland/X11, associação do diálogo à janela, responsividade ao salvar/exportar documentos grandes, tablet, tecnologia assistiva, corpus de screenshots e gates do MVP inteiro permanecem sem validação. Salvar/exportar ainda executa sincronicamente após a confirmação do destino. A API atual de rfd não distingue falha do seletor de cancelamento; a entrada manual continua disponível. Relatórios de degradação, seletor nativo para exportação, autosave/recovery, impressão, trabalho completo dos tokens claro/escuro e os demais itens da auditoria permanecem abertos.

## Mapeamento do Atlas

| Contrato | Implementação |
| --- | --- |
| 08.2 shell e abas | Router comum de arquivos, identidade do fechamento, entradas de teclado |
| 08.18 diálogos | Modais de configuração, destinos e fechamento/exportação |
| 08.10 / 08.29 exportação e fluxo | Destino efetivo, DPI/escopo do PNG, explicações de capacidades |
| 08.16 UI gauntlet | Interação focal headless Freya e testes de artefato PNG |
| 09.16 localização | Catálogo comum canônico EN/PT dos fluxos |

Decisão: [ADR-007](/pt/developers/adr/ADR-007-desktop-file-workflows).

Next UI wave / próxima etapa de UI: [Rascunhos estáveis de edição de objetos](/pt/developers/uiux-object-edits).
