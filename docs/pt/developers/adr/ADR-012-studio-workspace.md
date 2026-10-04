# ADR-012: Workspace Studio compacto e controles desktop semânticos

**Status:** Contrato de implementação aceito; evidência de execução separada.
**Data:** 2026-10-04
**Escopo:** Milestone Required (MVP); base desktop compartilhada com fluxos V1 Required.

## Contexto

A interface misturava idiomas, concentrava os painéis em um único inspetor, exibia ferramentas em duas colunas como uma linha horizontal e apresentava ícones pequenos sem identidade de teclado. Novas abas podiam herdar uma câmera com tamanho incorreto. O navegador desenhava caixas delimitadoras em vez da arte composta. Amostras substituíam a espessura do traçado e alteravam preenchimentos ignorados em raster. Algumas preferências anunciavam GPU ou simulavam tarefas sem representar a implementação real.

A referência é o [Affinity Studio atual](https://www.affinity.studio/graphic-design-software) e a [anatomia oficial do workspace](https://affinity.help/photo2/en-US.lproj/pages/Workspace/interface.html): interface neutra e compacta, seletor de Studio, controles contextuais, ferramentas à esquerda, arte central, grupos independentes de cor e inspeção à direita e retorno contextual abaixo. Petunia mantém marca, recursos, capacidades e limite de interação próprios. Não é criado um Studio Layout fictício.

## Decisão

1. A interface de produção compõe menus, barra Vector/Pixel, abas de documentos, contexto com rolagem horizontal, ferramentas com uma/duas colunas e rolagem vertical, canvas, docks auxiliares opcionais, Studio direito limitado e barra de status. Cor/Amostras permanecem disponíveis acima das abas Camadas/Propriedades/Cores/Histórico/Navegador/Tarefas. O grupo superior e todo o Studio podem recolher independentemente. Painéis auxiliares reservam as duas dimensões do canvas. Divisores compartilhados capturam o arraste pela janela, aceitam setas (Shift amplia o passo) e cancelam o arraste com Escape. Podem abrir/fechar após iniciar sem alterar a ordem dos hooks. A largura solicitada sobrevive ao redimensionamento; a largura efetiva reserva espaço mínimo para o canvas.
2. Controles Studio compartilhados possuem nome acessível, ativação por teclado, estados selecionado/desabilitado, foco visível, alvos de ponteiro e ícones semânticos próprios. Tipografia, cores e espaçamento ficam no tema. Ícones preenchidos usam contorno quando não existe um equivalente preenchido. EN e pt-BR compartilham catálogo e respondem ao idioma ativo. Aparência fica nas Preferências, com restauração do workspace.
3. Linhas de camada usam ObjectId e SessionIdentity para recolhimento. Ocultar descendentes não remove conteúdo. Seleção aditiva e controles independentes de nome/visibilidade/bloqueio preservam a seleção. Mutações passam pelos comandos da bridge; visibilidade de painéis e câmera são estado de visualização.
4. A roda compacta lê preenchimento/traçado real ou o pincel raster e aceita ponteiro e setas. Amostras e editores RGB/hex/Lab/spot compartilham o alvo. Cor vetorial aplica atomicamente aos objetos vetoriais selecionados e desbloqueados, preservando espessuras. Cor raster altera o pincel real, preserva alpha e limpa tinta CMYK literal anterior ao escolher RGB. CMYK nativo e prévias ICC mantêm ADR-011. Favoritos guardam a cor atual no estado da sessão do aplicativo; persistência em disco e bibliotecas spot certificadas não são anunciadas.
5. O navegador reutiliza fontes imutáveis e o worker de composição existente, incluindo texto, imagens, raster e efeitos. Desenha a área da câmera principal e move a vista por ponteiro/teclado sem alterar zoom ou revisão do documento. Épocas de apresentação publicam quadros dos workers, mudanças de câmera e sobreposições de seleção/texto apesar do cache por igualdade de callbacks. Sessões novas/abertas enquadram a prancheta uma vez após dimensionar o canvas; voltar à aba preserva sua câmera. Workspaces de baixo nível podem desativar o enquadramento inicial.
6. Novo documento aceita nome, largura, altura, sangria e margens exatos, com unidades explícitas em pontos, vírgula decimal e validação. Valores inválidos preservam o rascunho sem criar aba/histórico. Presets e orientação alteram os mesmos campos. Formulários usam Popups limitados; pesquisa e inspetores têm rolagem. Exportação só apresenta limites pertinentes ao formato. Preferências descrevem o renderizador real e Tarefas mostra jobs reais.
7. Atalhos do canvas respeitam foco em campos e teclas sem modificadores pertencentes a botões/abas/árvore. Modais continuam isolando atalhos de edição. Motivos de capacidades indisponíveis, falhas dos workers e retorno de arquivos ficam visíveis; funcionalidades ausentes não recebem estado de sucesso.

## Mapeamento do Atlas

| Contrato | Implementação |
| --- | --- |
| 08.1 / 08.2 / 08.3 | Tokens, shell compacto, grupos Studio, redimensionar/recolher/restaurar |
| 08.5 / 08.14 | Controles acessíveis, foco e propriedade dos atalhos |
| 08.6 / 08.17 | Enquadramento por sessão, navegador real, recolhimento e cor semântica |
| 08.18 / 08.19 | Novo documento numérico, modais limitados, retorno e unidades reais |
| 08.12 / 09.16 | Preferências de aparência e catálogo EN/pt-BR |
| 08.16 / 14.7 | Evidência headless da interface de produção, pixels e capturas |

## Consequências e verificação

Não muda o schema nem as dependências GUI do domínio. Raster, texto, ICC, exportação e undo mantêm seus adapters. Testes focados e gauntlets aplicáveis executam **após a implementação**, conforme solicitado. O [registro de execução](/implementation/uiux-studio.json) reúne comandos, resultados, capturas e hashes reais. A [implementação Studio](/pt/developers/uiux-studio) registra auditoria e aceitação delimitada.

Este incremento não certifica release completo MVP/V1 nem paridade de funcionalidades com Affinity. Multisseleção numérica mista, docking inteiramente configurável, persistência de favoritos, tema claro/sistema, alto contraste completo, validação física com leitor de tela/IME/tablet/HiDPI, usabilidade representativa e impressão/tipografia profissionais mantêm o escopo do roadmap. Asserções headless de acessibilidade não certificam tecnologia assistiva real. Prumo estava indisponível; o registro constitui o microcontexto local de objetivo/contrato/evidência.
