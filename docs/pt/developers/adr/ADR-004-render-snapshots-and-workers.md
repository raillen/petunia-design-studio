# ADR-004: Snapshots vetoriais imutáveis e workers limitados

**Status:** Contrato aceito; implementação aguardando validação  
**Data:** 2026-10-01  
**Escopo:** Milestone Required (MVP)

**Extensão de imagens:** [ADR-005](/pt/developers/adr/ADR-005-immutable-image-assets) fornece os adaptadores de imagem/cache limitado e composição CPU antes indisponíveis aqui. As menções a imagens abaixo descrevem a onda original ADR-004; glyph runs, apresentação GUI assíncrona da cena e workers de tiles continuam pendentes.

**Extensão de texto/apresentação:** [ADR-006](/pt/developers/adr/ADR-006-shaped-text-and-canvas-preview) acrescenta contornos com shaping e apresentação CPU comum no canvas. Pendências abaixo registram a onda original deste ADR; escopo/evidência atual está no registro de execução.

## Contexto

O compositor CPU anterior pintava bounds, amostrava uma única cor de gradiente e aproximava sombras por retângulos. A travessia plana perdia transforms dos ancestrais e isolamento de grupos. O PNG alocava uma imagem da origem do pasteboard até a prancheta para recortá-la depois; origens negativas perdiam arte e origens positivas distantes multiplicavam a memória. O gerenciador registrava estados de jobs sem executar trabalho.

## Decisão

`RenderScene`/`RenderSurface`/`RenderNode` são snapshots imutáveis e reconstruíveis, com identidades estáveis, geometria local avaliada, transforms mundiais compostos pelos ancestrais, ordem canônica de filhos e limites visuais. `Scene` permanece um resumo headless legado de inventário, não uma representação da arte. Paths fonte continuam editáveis. Fontes de imagens codificadas são compartilhadas por buffers Arc imutáveis, preservando o formato nativo existente de arrays de bytes e evitando cópias em snapshots/histórico/duplicação; isso ainda não implementa recursos binários no pacote nem cache de imagens decodificadas. Nenhum snapshot/worker recebe documento mutável ou tipos de toolkit gráfico.

O backend CPU de referência usa o rasterizador `tiny-skia` 0.11.4, já presente no lockfile e independente da GUI. A inspeção de painter, máscaras, shaders e stroker orientou o adaptador. Fills usam cobertura even-odd, coerente com a política atual de geometria/hit test. Caps, joins, dashes e alinhamento center/inside/outside usam paths reais; inside/outside exige todos os contornos fechados. Paints conservam seus descritores atuais no frame do pai, convertidos explicitamente ao frame local. Frames locais persistentes de referência para paints coloridos ainda exigem migração própria; frames de modificadores não migram paints implicitamente.

Objetos/grupos compõem em intermediários transparentes antes de aplicar opacidade total e transparência espacial uma única vez. Os dezesseis blends do documento mapeiam ao backend. Máscaras vetoriais rasterizam paths; máscaras alpha amostram alpha renderizado; luminância usa sRGB linear multiplicado por alpha. Clip groups exigem exatamente uma máscara designada. Contêineres atuais são isolados; pass-through futuro exige contrato explícito no documento.

Pixels permanecem premultiplicados na cobertura, composição, máscaras e convolução. A API retorna RGBA8 direto. Preview de cor básico, interpolação de gradientes e composição usam sRGB codificado; ajustes tonais respeitam o contrato existente de RGB linear normalizado, com decoding/encoding sRGB em torno da avaliação. Isso não implementa gerenciamento ICC nem prova CMYK.

Gaussian blur usa kernel separável normalizado, truncado a três desvios padrão, com amostras transparentes além da fonte. Os campos atuais de radius significam sigma em pontos do documento. Cada eixo escala separadamente para pixels. Drop shadow colore e desfoca a silhueta composta atual, preservando o original acima. Alcances dos efeitos ordenados se acumulam. Intermediários cobrem limites visuais intersectados com halo de convolução, preservando a fonte que contribui para bordas suaves ao recortar viewport/região dirty. Curvas Fritsch–Carlson preparadas calculam tangentes uma vez e fazem busca binária de segmentos sem alocação por amostra; não usam aproximação por tabela nem recalculam tangentes por pixel.

`RenderLimits` tem defaults de 16.777.216 pixels de saída, 256 MiB de intermediários/saída/scratch raster simultaneamente reservados, sigma máximo de 128 pixels e profundidade de composição de até 128. Alocações usam tamanhos verificados e reservas falíveis. Admissão de snapshots também limita objetos, profundidade e complexidade individual de paths/formas paramétricas. Isso ainda não constitui quotas completas de memória/CPU contra documentos hostis: fontes/geometria compilada, aparência e trabalho total exigem orçamentos adicionais, assim como caches de imagens/fontes.

`RenderRequest` mapeia uma região mundial explícita diretamente às dimensões de saída. Exportação usa DPI/72 pixels por ponto, independente da origem, com default de 72 DPI. PNG registra metadados sRGB e pHYs de densidade física (arredondados a pixels inteiros por metro), usa o backend e retorna motivo de capability antes de escrever arte incompatível. Glyph runs, recursos de imagem decodificados, modificadores geométricos em grupos compostos, inner shadow, sharpen e noise continuam indisponíveis neste backend. O compositor de compatibilidade agora retorna `Result`; consumidores tratam erros. Objetos sem shape não inventam retângulos.

`RenderSurface::damage_to` mantém limites anteriores e novos, inclusive efeitos removidos, propagando dependências de máscaras/ancestrais. Reordenação de roots invalida conservadoramente. O helper antigo baseado apenas em ChangeSet permanece aproximado e explicitamente inadequado para renderização hierárquica/com efeitos. Repaint dirty prepara todos os pixels antes de copiá-los e compõe sobre o backdrop original quando nenhum background substituto é solicitado.

`JobExecutor` possui número fixo de threads, fila limitada, tokens cooperativos, progresso e uma vaga de resultado por proprietário. Panics falham o job sem encerrar o worker. Resultados carregam a revisão fonte; proprietários rejeitam revisões obsoletas antes de apresentar/publicar. Drop do handle cancela o trabalho. Drop do executor sinaliza cancelamento sem bloquear a UI; shutdown-and-join explícito serve ao teardown com tarefas cooperativas. Renderização verifica cancelamento entre nós e linhas de pixels/convolução; uma chamada limitada da biblioteca de paths não é interrompida no meio. `schedule_surface_render` conecta o agendamento da aplicação à cena imutável. Apresentação na UI, caches de imagens/fontes e workers de tiles persistentes ainda precisam de integração.

## Consequências e evidência

Os caminhos novos de snapshot/backend/worker estão em implementação, **sem gate MVP aprovado**. O usuário pediu explicitamente executar testes e validações somente após implementar todas as features do MVP. Fontes de regressões estão incluídas sem execução; build, Clippy, fronteiras, comparação de corpus, documentação e aceite Linux/produto continuam pendentes. Os 661 testes anteriores descrevem apenas o commit da base. Commits desta fase em rascunho adiam CI automático; gates finais precisam executar antes de tornar o PR pronto para revisão.

A arquitetura segue [W3C Compositing and Blending](https://www.w3.org/TR/compositing-1/) e o modelo Porter–Duff ([DOI do paper de 1984](https://doi.org/10.1145/800031.808606)); são referências algorítmicas, não nova evidência independente de aceite. O algoritmo existente de curvas é interpolação cúbica monotônica Fritsch–Carlson ([DOI do paper de 1980](https://doi.org/10.1137/0717021)). Esta continuação inspecionou o repositório e o fonte local do rasterizador; não declara novas leituras completas desses papers nem equivalência visual/de benchmark a programas concorrentes.
