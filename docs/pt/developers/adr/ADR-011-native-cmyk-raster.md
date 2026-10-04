# ADR-011: Raster CMYK nativo, recursos ICC e intercâmbio preservando tintas

**Status:** Contrato de implementação aceito; evidência de execução separada.
**Data:** 2026-10-03
**Escopo:** V1 Required; o comportamento existente do MVP permanece coberto pelos gates finais.

## Contexto

O ADR-010 estabeleceu recursos ICC reais e prova da composição RGB. RGBA não armazena quatro tintas de processo e transparência independente. A admissão TIFF pela fachada genérica de imagens converte CMYK em RGB; tratar esse resultado como tinta autoral perderia as separações antes da edição.

## Decisão

1. `Cmyka8` e `Cmyka16` armazenam C/M/Y/K/alpha reto em cinco canais inteiros; tiles de 16 bits usam little-endian. Tinta nunca é pré-multiplicada. Acesso RGB a tinta nativa retorna erro. Tinta oculta transparente sobrevive ao commit; branco e ciano puro com K=0 permanecem visíveis. Aplicam-se limites de dimensões/tiles/saída densa, padding zero e compartilhamento copy-on-write. Amostras nativas exigem ICC CMYK de saída validado; descritores vazios são resolvidos antes da publicação do pacote binário.
2. Atribuição altera o perfil e compartilha os pixels exatos. Conversão é operação explícita e imutável CMYK→CMYK via LittleCMS, sem passagem RGB; identidade de perfil preserva exatamente as amostras. Alpha fica fora do CMM. Um transformador pertence à operação/worker e processa lotes de linhas. Exibição usa cache derivado ICC sRGB de até oito tiles (2 MiB), sem alterar recursos autorais. Cancelamento rejeita saídas incompletas. Não se afirma suporte a DeviceLink ou preservação de geração de preto específica da impressão.
3. Schema 6/índice 3 armazena ICC da camada como recurso binário SHA-256, deduplicado com os perfis de superfície e contado nos limites de admissão/histórico. O descritor não contém arrays ICC ou tiles inline. Schemas 1–5 e índices 1–2 continuam legíveis para seus layouts; contratos antigos não admitem CMYKA. Hash inválido, recurso extra, perfil ausente ou descritor contraditório falham na admissão.
4. Pintura e preenchimento operam nos canais de processo literais com alpha independente. Cor RGB do pincel é convertida pelo perfil da camada uma vez no início; tinta CMYK explícita evita essa conversão. Borracha altera somente cobertura. Pressão, transformação, seleção, cancelamento, sessão/revisão capturada e undo por gesto continuam no limite Action/Command. Criar uma camada nativa de 16 bits exige perfil de impressão da camada/superfície. Importação TIFF cria camada editável em worker; o seletor de perfil captura a identidade original do objeto.
5. TIFF nativo lê/grava processo CMYK 8/16-bit, orientações 1–8, ICC embutido e alpha não associado explícito (ou entrada opaca de quatro canais). Usa o codec diretamente. Saída tem Deflate sem perdas em strips limitados, DPI e publicação atômica. ICC ausente/não adequado, alpha associado, layout planar, múltiplas páginas/tintas não processuais e precisão não suportada são rejeitados. Exportação contém **uma camada autoral**, antes de máscaras, efeitos, opacidade e composição de página; não é exportação de separações de página.
6. PDF regular incorpora as quatro tintas originais com ICC original e máscara alpha independente, preservando 8/16-bit. Imagem codificada CMYK mantém a mesma semântica. O backend atual exige um perfil CMYK comum entre superfícies incluídas, camadas nativas e imagens codificadas nativas; perfis diferentes bloqueiam a exportação com motivo para conversão explícita. O perfil original é incorporado como espaço CMYK do documento, incluindo versões ICC que o backend omitiria silenciosamente de imagens individuais. SVG/intercâmbio somente RGB recebe derivada de exibição explícita. PDF estrito rejeita efeitos de página não suportados; rasterização RGB autorizada reporta `CMYK_PAGE_CONVERTED_TO_RGB`. PDF/X-4, conformidade OutputIntent, paridade de spot/overprint e prova direta da página CMYK continuam indisponíveis.
7. `ink_coverage` reporta soma literal de tintas dos pixels visíveis e máximos por canal. O teto TAC é política opcional fornecida pelo chamador para a impressão, nunca constante universal. Separações por camada contêm tinta vezes alpha. São ferramentas de inspeção, sem composição de chapas considerando overprint. A prova existente da composição RGB é bloqueada para raster/vetor/imagem CMYK autoral para evitar apresentá-la como prova nativa fiel.
8. CLI realiza operações reais com `cmyk-import`, `cmyk-export-layer`, `cmyk-inspect`, `cmyk-assign`, `cmyk-convert` e `export-pdf`. Documento ambíguo exige `--object ObjectId:N`. Sem argumentos permanece o fluxo de conformidade. Escritas usam comandos do domínio e saída atômica. Strings EN/pt-BR e razões das capacidades descrevem o escopo suportado.

## Mapeamento do Atlas

| Contrato | Implementação e limites |
| --- | --- |
| 09.6 / 10.9 raster | Tiles COW de cinco canais 8/16-bit; pincel/borracha/preenchimento nativos; rascunhos imutáveis |
| 09.3 commands | Criação/atribuição tipadas, admissão por worker com precondições e undo/redo |
| 09.9 color / V1-B | Atribuição/conversão/exibição ICC; inspeção TAC e separação de camada |
| 09.13 persistence / 15.A migration | Schema 6/índice 3 com perfis binários; leitura RGB/gray legada |
| 08.17 panels / 08.18 workflows | Criar camada, definir tinta, perfil com objeto capturado e importação TIFF editável |
| 08.29 / V1-C interchange | TIFF de camada e PDF regular CMYK ICC; degradação RGB explícita |
| 14.1 / 08.16 validation | Regressões de amostras/alpha/ICC, CLI, interação Freya e leitura independente Poppler |

## Consequências e evidências

Testes focados cobrem round trips exatos de tiles/TIFF/pacotes, amostras de 16 bits/ocultas, orientação, recursos inválidos, limites, imutabilidade de fonte/alpha, cancelamento/atomicidade, histórico de gestos, CLI, bloqueio de prova nativa e interação do painel. O CI também executa conferência independente pypdf/Poppler das amostras de tinta, máscara alpha e bytes do ICC original para camadas editáveis e imagens codificadas. A validação ocorre somente após implementar, conforme pedido. O [registro de execução](/implementation/native-cmyk-v1.json) contém resultados reais; contagens de fontes/testes não são evidências de execução.

**A V1 completa e a aceitação do release MVP permanecem abertas.** Composição/prova direta da página nativa, paridade overprint/spot, política DeviceLink/preto, PDF/X-4, tipografia avançada e aceitação representativa de impressão/hardware/usuários continuam V1 Required. O perfil sintético próprio é fixture de engenharia, não caracterização de uma impressora de produção.

Referências primárias: [TIFF 6.0](https://download.osgeo.org/libtiff/doc/TIFF6.pdf), [tags libtiff](https://gitlab.com/libtiff/libtiff/-/blob/master/libtiff/tiff.h), [codec TIFF](https://docs.rs/tiff/0.11.3/tiff/), [API LittleCMS](https://www.littlecms.com/LittleCMS2.16%20API.pdf), [errata PDF ICCBased aprovada ISO, 8.6.5.5](https://pdf-issues.pdfa.org/32000-2-2020/clause08.html).
