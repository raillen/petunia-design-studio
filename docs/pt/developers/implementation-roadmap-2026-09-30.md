# Plano de implementação — MVP, V1 e versões futuras

Este plano parte da baseline `73447647a34f5b4be209415c36e6965c5f71b9b9` e dos 37 registros do [dossiê](./audit-2026-09-30). É uma proposta de escopo e sequência; não declara tarefas implementadas. **MVP:** editor vetorial e bitmap com ferramentas principais realmente utilizáveis. **V1:** produção pessoal e profissional, incluindo **CMYK verdadeiro com ICC e PDF profissional**. **Futuras:** recursos especializados e diferenciação, depois de correção, estabilidade e uso real.

Linux é a plataforma primária. A portabilidade fica nos contratos de plataforma e recursos; outras plataformas não devem consumir o esforço necessário para concluir os fluxos Linux. Manter Freya/Skia enquanto um teste concreto não demonstrar bloqueio insuperável; não iniciar outra migração de GUI por preferência.

## Regras de implementação e evidência

1. Toda alteração passa por Action→Command→DocumentMutator→ChangeSet, com commit atômico, revisão esperada e undo/redo. Preview de gesto não é documento paralelo persistente. Um gesto confirmado gera uma operação de histórico; Esc cancela sem resíduos.
2. Documento e recursos armazenam fontes editáveis. Effects, ajustes, máscaras e modificadores são ordenados e persistentes. Expand/Bake/Rasterize/Convert-to-Curves exigem operação explícita; imagem original não é sobrescrita por preview ou brush em camada separada.
3. Definir semântica e referência de resultado antes de otimizar. Preview, seleção, bounds, SVG/PNG/PDF partem da mesma cena avaliada. Cada backend declara fidelidade/capabilities, e perdas não passam silenciosamente.
4. Testes de propriedades, pixels e round trip atacam invariantes reais. Asserts dos probes da baseline observam bugs: convertê-los em regressões que esperam o resultado correto, sem perpetuar o erro.
5. Workers limitados recebem snapshots imutáveis e publicam resultados por Command com revisão esperada. Cancelamento, limites de memória e saída atômica são requisitos funcionais.
6. Cada entrega contém dono, status de escopo, fixtures, checks aplicáveis, documentação en-US/pt-BR, diagnóstico de indisponibilidade e evidência humana quando houver interação. Um painel renderizado ou handler chamado não conclui a ferramenta.

Status usados: **Milestone Required (MVP)**; **V1 Required**; **Post-V1 Candidate**; **Research**; **Open ADR**. O plano propõe promover ICC de POST_V1 para V1 Required, conforme pedido do usuário. Isso precisa de ADR e atualização de Atlas/authority map na implementação; este texto não muda silenciosamente os contratos antigos.

## Dependências e ordem

```mermaid
flowchart LR
  A[M0: correção, schema e mutação] --> B[M1: cena e renderer coerentes]
  B --> C[M2: ferramentas vetoriais e texto]
  B --> D[M3: pixels, máscaras e ajustes]
  C --> E[M4: arquivos, Linux e MVP]
  D --> E
  E --> F[V1-A: precisão, assets e UX profissional]
  E --> G[V1-B: ICC e CMYK]
  F --> H[V1-C: PDF/X-4 e interoperabilidade]
  G --> H
  H --> I[V1-D: validação e release]
  I --> J[Futuras: especialização e pesquisa]
```

M2 e M3 podem ocorrer em paralelo após os contratos de cena/recursos de M1, sem duplicar armazenamento ou composição. Trabalho de perfis/licenças pode começar cedo; a entrega CMYK depende dos contratos de cor e recursos, e PDF/X-4 depende de texto, composição e ICC.

## MVP — ferramentas principais completas

O MVP serve para criar logos, ilustrações simples, cartazes digitais, layouts multiprancheta e composições com pintura/edição bitmap básica. O usuário deve concluir, salvar, reabrir e exportar essas tarefas. O escopo de cor é RGB gerenciado básico com perfil de entrada/display e diagnóstico honesto; o documento já admite perfis e valores tipados para receber CMYK na V1. Não rotular a simulação atual como prova ICC.

| Entrega | Implementação e contratos | Dependência | Aceite obrigatório | Achados |
| --- | --- | --- | --- | --- |
| **M0.1 · Correções reproduzidas** | Resolver R-tree revision-zero; geração dos flats; A−∅; RDP degenerado; eraser; panic UTF-8; alpha duplicado e offset que descarta componentes | Baseline + regressões focais | Probes convertidos em testes do resultado correto, incluindo ordens de leitura/undo | F02, F03, F09, F11, F17–F20 |
| **M0.2 · Schema e validação** | GeometrySpace explícito, migração dos Path legados, IDs/referências/árvore válidos, finitude e limites; restringir writes diretos | Decisão de geometry frame | Paths criados e legados continuam editáveis; arquivos inválidos têm diagnóstico e não travam | F01, F22 |
| **M0.3 · Transação e histórico** | Commit com revisão esperada, replay atômico, COW/deltas, orçamento em bytes e savepoint; índices ID→objeto | M0.2 | Conflito não apaga alteração; falha de undo preserva estado; gesto único/cancelamento e memória limitada | F23–F26 |
| **M1.1 · Cena canônica** | DTO imutável: geometria local/world, paints tipados, glyph runs, recursos, grupos, clips, visual bounds; invalidar por dependência | M0.2–M0.3 | Câmera/seleção não recompõem geometria; descendentes invalidam com ancestor transform | F01, F03–F07, F25 |
| **M1.2 · Render de referência** | Skia consumindo cena; coverage real, antialias, gradients, strokes, blend/alpha, grupos isolados, masks e efeitos básicos | M1.1 | Corpus de pixels aprovado; clip vazio desenha zero; imagem/texto/gradient iguais em preview e PNG | F04–F07, F10–F11 |
| **M1.3 · Cache e workers** | Cache decoded/mip/font/geometry por bytes/geração; executor limitado, cancelamento real e publicação por Command | M0.3 + M1.1 | Sem I/O/decode em frame inalterado; trabalho grande mantém interação; job cancelado não vira Completed | F06, F25, F27–F29 |
| **M2.1 · Edição vetorial** | Seleção/multisseleção, mover/escala/rotação, nós/cusp/smooth, pen/pencil, shapes, fills/strokes/gradients, booleans, grupos, ordem e masks | M1 | Cada ferramenta passa fluxo criar→editar→undo→save/reopen→export, incluindo zoom e transforms | F01–F05, F18–F21 |
| **M2.2 · Texto básico correto** | Texto artístico e caixa multilinha; família/tamanho/peso/itálico/alinhamento; clusters/grafemas/bidi/fallback; caret/IME; glyph runs comuns | M1.1 + decisão de shaping | Poster com acentos, ligaturas, fallback e multilinha; cursor/seleção corretos; fontes ausentes diagnosticadas | F07, F32 |
| **M3.1 · Camadas de pixels e seleção** | PixelLayer/MaskLayer e TileStore persistentes; brush/eraser, retângulo/elipse/lasso, preenchimento básico, crop e transform de layer | M0.3 + M1 | Pintura real sob seleção/máscara; strokes atômicos; pixel layer round trip; originais preservados | F08–F10, F28 |
| **M3.2 · Ajustes não destrutivos** | Levels/Curves, exposição/brilho/contraste, hue/saturation e blur/shadow básicos em cadeia ordenada; máscara e toggle/reordenação | M3.1 + M1.2 | Alterar/desligar/reordenar após reabertura; preview/export coincidem; nenhuma sobrescrita da fonte | F04, F06, F11 |
| **M4.1 · Arquivos e saídas** | PTND com recursos binários, saves exclusivos/atômicos e recovery; import PNG/JPEG/TIFF RGB e subconjunto SVG documentado; PNG em DPI escolhido e SVG fiel ao escopo | M1–M3 | Falha preserva arquivo anterior; origens negativas/distantes corretas; export não usa retângulos substitutos | F11, F14–F15, F27, F37 |
| **M4.2 · Linux e UX** | Wayland/X11, HiDPI, pen com pressão real, teclado/foco/IME, clipboard nativo, diálogos/portais, acessibilidade e l10n; diagnóstico de capabilities | M2–M3 | Quatro tarefas MVP completadas; hardware/backend registrados; zero simulação na UI de produção | F29, F32–F34 |
| **M4.3 · Gate e pacote MVP** | Toolchain testado fixo, CI real, corpus visual/arquivos, limites e benchmark controlado; pacote Linux e smoke de instalação | Todas as anteriores | P0/P1 MVP resolvidos ou escopo explicitamente retirado sem afetar ferramentas principais; nenhum gate stub vendido como sucesso | F34–F36 |

PDF profissional é V1 Required. Se um PDF simples for oferecido no MVP, primeiro corrigir dimensões/seleção e substituir o renderer aproximado; declarar o subconjunto real, fontes/imagens preservadas e preflight de perdas. Caso contrário, manter a capability indisponível com motivo. A presença do exporter atual não autoriza habilitá-lo como saída fiel.

### Inventário funcional das ferramentas

Cada linha significa ferramenta integrada com seleção, histórico, persistência e exportação aplicável. Preferências e controles só aparecem habilitados se tiverem efeito real.

| Família | MVP — Milestone Required | V1 Required | Post-V1 Candidate |
| --- | --- | --- | --- |
| Navegação/seleção | Pan/zoom/fit, select/marquee, multisseleção, lock/hide, ordem, layers | Seleção por atributo, busca e isolamento avançado, x-ray | Seleção semântica e assistida |
| Geometria | Move/scale/rotate, numeric transform, align/distribute, snapping/guides | Shear/perspective, pontos de referência, precisão/medidas avançadas | Constraints, layout paramétrico completo |
| Curvas/shapes | Pen, pencil, node, rect/ellipse/polygon/star, boolean e convert explícito | Knife/scissors, shape builder, contour robusto, compound/holes, live boolean | Mesh/envelope complexo, calligraphic/vector brush avançado |
| Aparência | Solid/linear/radial gradients, stroke widths/caps/joins/dashes, básico blend, group/mask, shadow/blur | Múltiplos fills/strokes completos, padrões, estilos, variável-width básica, clip avançado | Mesh gradients, procedural paints, materiais |
| Texto | Artístico/caixa, multiline, estilos básicos, shaping correto/IME | Texto em caminho, paragraph/character styles, OpenType, links de caixas e fontes variáveis com escopo testado | Composição editorial longa e features linguísticas especializadas |
| Pixels | Brush/eraser, fill, seleção rectangle/ellipse/lasso, crop, mask e ajustes básicos | Clone/heal, feather/refine, wand, channels, filtros live, brush engine com spacing/texture/tilt | Liquify, smudge/wet paint, segmentação e filtros avançados |
| Documento | Múltiplas pranchetas, unidades, guides, save/reopen, undo/redo, recovery | Bleed/margins/pages, asset linking, components/symbol instances e bibliotecas reutilizáveis | Imposição editorial, templates/data merge complexos |
| Cor | RGB gerenciado básico, swatches tipados, pipeta fiel | ICC/CMYK real, Lab, proof, monitor, intents/BPC, spot/registration, separações/TAC | Multicanal especializado, spectral/DeviceN avançado, HDR/OCIO |
| Interoperabilidade | PTND, SVG no escopo, PNG/JPEG/TIFF RGB input, PNG/SVG output | PDF/X-4, JPEG/TIFF 8/16-bit gerenciados, CMYK TIFF, SVG interoperável completo no escopo contratado, import PDF/PSD limitado com relatório | Import amplo AI/PSD/complex PDF, recursos cloud e round trip especializado |
| Produtividade | Atalhos, properties/context toolbar, layers, feedback e export preview real | Presets, macros/CLI batch, find/font/resource manager, templates e espaços salvos | Marketplace, colaboração e automação assistida |

### Gate de saída do MVP

- Quatro projetos de tarefa — logo, poster, pintura com mask e múltiplas pranchetas — concluídos por usuários sem intervenção de desenvolvedor. Registrar erros e tempo; incluir pessoa usando teclado e tecnologia assistiva.
- Criação/edição de todas as ferramentas MVP conserva identidade e efeitos editáveis após save/reopen, undo/redo e cancelamento. Fixture transformada/agrupada tem a mesma aparência nos exports suportados.
- PTND sobrevive a saves concorrentes, falta de disco, cancelamento e interrupção; recovery oferece última versão válida sem substituir original silenciosamente.
- Performance e limites medidos no hardware declarado, com cenários do plano; testes de tempo debug não substituem benchmark release.
- CI estrito, docs sincronizados e capability matrix distinguem integrado de validado. Todos os P0 MVP fechados; P1 restante exige decisão de escopo com risco concreto, sem manter ferramenta principal parcialmente funcional.

## V1 — uso pessoal e profissional, CMYK e PDF

### V1-A · Precisão e fluxos profissionais

Completar ferramentas da coluna V1: paths compostos/holes e edição avançada, contour/knife/shape builder testados, paints múltiplos, stroke/alinhamento consistente, text styles/OpenType/on-path/frames vinculados, brushes e filtros live, seleção refinada, channels, linked assets com atualização segura e symbols como instâncias editáveis reais. Não confundir biblioteca de presets de shapes com component instances.

Estender formato para fontes/recursos/ICC e manter compatibilidade por migração explícita. Font manager informa substituições e direitos de embedding; preflight identifica imagem ausente, baixa resolução efetiva, texto overset e perfil incompatível. Adicionar presets e export batch com progresso/cancelamento reais. Plugins entram apenas com quotas e testes de isolamento; adaptador MCP interoperável precisa de cliente independente, não apenas métodos ptnd.* locais.

Aceite: projetos reais de identidade visual, cartaz, ilustração, imagem retocada e peça impressa; atalhos e painéis coerentes, multisseleção com valores mistos, unidades editáveis, capability reasons e relatórios recuperáveis. **Achados:** F04–F07, F15, F20–F21, F28–F33.

### V1-B · CMYK verdadeiro e gerenciamento ICC

**Definição operacional:** CMYK é dado do documento com canais C/M/Y/K preservados e perfil real; display/proof são derivações. Alterar monitor, proof ou zoom não altera tinta armazenada. DeviceCMYK no PDF e uma fórmula RGB↔CMYK não satisfazem esse requisito.

1. **Modelo:** ColorValue tipado + ProfileId/hash + unidades/faixas; swatches process/spot/registration; pixels com formato de canais explícito, incluindo CMYK(A), Gray e RGB, 8/16-bit onde aplicável. Arquitetura deve separar quatro canais de tinta de alpha e da representação de display. Não forçar todos os layers para RGBA na persistência.
2. **Registro de perfis:** carregar bytes ICC v2/v4, validar compatibilidade com espaço/canais e incorporar hash/metadados no pacote. Instalar perfis depende de licença; nomes de perfis não substituem arquivos. Ao abrir sem perfil, escolher Assign, Convert ou preservar sem perfil com aviso; nunca adivinhar silenciosamente.
3. **CMM:** avaliar LittleCMS e alternativas Rust por capacidade real. Preferir integração validada do motor existente a implementar CMM do zero. Chave do transform inclui source/destination/proof hash, formatos, intent e flags; cache limitado, transform por tile e política de threads comprovada. LittleCMS é referência diferencial, com entradas e tolerâncias declaradas.
4. **Transformações:** PCS D50 Lab/XYZ, adaptação cromática quando necessária, quatro intents, BPC, convert vs assign, conversão CMYK→CMYK e preservação de preto. Separação/GCR/UCR/TAC segue perfil/DeviceLink/política explícita; não aplicar um teto universal ou fator 0,98.
5. **Soft proof:** source→press→monitor, proof intent, paper white/black, gamut warning e BPC. Linux precisa de obtenção/configuração de perfil do monitor e comportamento documentado por backend; tela não calibrada não vira referência colorimétrica por software.
6. **Separações e preflight:** visualizar C/M/Y/K e spots, distinguir preto puro/rich black, conferir TAC com limite do destino, overprint/knockout e registration. Guardar preview de spot separado da identidade e dos valores de tinta.
7. **Validação:** patches cromáticos, gradientes, extremos de tinta, CMYK puro, black preservation, perfis diferentes, profile assignment sem alterar canais, undo/reopen e export sem round trip RGB. ΔE00 só com implementação e dados de referência; limite deve considerar precisão e perfil, não um número universal sem contexto.

**Gate:** diferenças mensuradas contra CMM independente; dados CMYK e spots preservados; proof acompanha perfil de destino; TIFF/PDF e pacote incorporam os perfis necessários. **Achados:** F16–F17, F10, F13. **Dependências:** M0 schema, M1 paints/composição, M3 recursos e V1-A texto/recursos.

### V1-C · PDF profissional e interoperabilidade

Prioridade **PDF/X-4**, por suportar transparência e gerenciamento de cor. PDF/X-1a pode ser acrescentado como preset restritivo com flattening controlado quando houver demanda; não substituir X-4 por uma falsa promessa de compatibilidade.

- Página por superfície selecionada: MediaBox/TrimBox/BleedBox/CropBox, unidades, origem local, sangria/margens e marks opcionais. Páginas excluídas não entram; ordem e metadata são opções reais.
- Geometria e appearance avaliadas: full world transforms, fill rules, paints/gradients, strokes, clips, grupos isolados e transparência/blend. Rasterizar apenas efeito/capability sem representação fiel, com DPI, bounds, cor e aviso explícitos; nunca usar retângulos substitutos.
- Texto: glyph placement idêntico ao layout, fonts embedded/subset, licença verificada, ToUnicode e fallback diagnosticado. Convert-to-curves é escolha explícita; manter texto selecionável quando possível.
- Imagens: embedding real, resolução efetiva, clipping/máscaras/alpha, compressão com política e perfil. JPEG/TIFF/16-bit exigem caminho validado; não copiar pixels Straight como Premul.
- Cor: OutputIntent ICC válido para destino, RGB/CMYK gerenciados conforme preset, tintas spot/Separation e registration, overprint/knockout e separações corretas. DeviceN especializado pode ser futuro, preservando a capacidade V1 de spot necessária ao escopo profissional.
- Preflight: fonts/resources/profiles ausentes, overset, RGB não permitido no preset, TAC, resolução, sangria, transparência e perdas. Export bloqueia erro obrigatório e lista aproximação antes de escrever; relatório corresponde ao arquivo gerado.
- Verificação independente: parse/page boxes/fonts com ferramentas externas, rasterização por pelo menos um motor diferente do exporter, visual diff e inspeção de separações/RIP. Uma leitura bem-sucedida por pdfinfo não certifica PDF/X-4; usar validador que suporte esse padrão e registrar resultado. Teste com gráfica/perfil autorizado confirma uso real.

**Gate:** piece impressa multipágina com CMYK/spot/preto puro, texto e imagens, transparency/bleed; equivalência visual, canais e preflight; falhas/cancelamento conservam arquivo anterior. **Achados:** F12–F17, F27, F37. Dependências: V1-A e V1-B.

### V1-D · Release profissional

Suite de compatibilidade versionada com arquivos reais autorizados e corpus sintético; performance de sessões longas, projetos grandes, export batch, crash/recovery e hardware Linux. Finalizar assinatura/packaging aplicável, SBOM/licenças de dependências/fonts/profiles/assets, política de migrações e testes de instalação/update. Teste X11 e Wayland, software fallback e GPUs representativas; accessibility/l10n e tablet com pressão real.

Gate de saída: todos os V1 Required integrados e validados; CMYK/PDF sem aproximação oculta; nenhuma tarefa demonstrativa; docs/ADRs/Atlas reconciliados e limits publicados; usuários profissionais concluem projetos representativos. Uma falha descoberta reabre o gate correspondente, sem reclassificá-la como PROVEN por existir um teste de dispatch.

## Versões futuras — o que fica fora e ideias

| Horizonte/status | Recursos | Pré-requisitos e critério de seleção |
| --- | --- | --- |
| **V1.x · Post-V1 Candidate** | Mesh gradients, envelope/mesh warp, blends/repeats, pattern tools, vector brushes avançados, smart guides/constraints avançados | Renderer e edição corretos; demanda de projetos reais; preservação live e round trip |
| **V1.x · Post-V1 Candidate** | Smudge/liquify, brush textura/rotação avançada, frequência/retouch especializado, filtros e seleções refinados adicionais | Tile DAG, halos, undo e benchmark; preservar fonte/mascara/params |
| **V2 · Post-V1 Candidate** | Data merge completo, barcode/template systems, multi-page editorial, master pages, imposition | Texto/recursos/PDF professional estáveis; preflight por resultado e licenças |
| **V2 · Post-V1 Candidate** | Importação ampliada PSD/AI/PDF, round trip de subsets, novas saídas e print presets | Especificações/licenças e corpus; publicar o que vira editable, rasterized ou unsupported |
| **V2 · Research** | Renderer GPU compute, tiles virtuais, tessellation/cache avançados, SIMD especializado | Profiling mostra bottleneck; igualdade com CPU reference, fallback e orçamento VRAM |
| **V2 · Research** | HDR/float, wide-gamut avançado/OCIO, DeviceN multicanal e recursos espectrais | CMM/precision pipeline e hardware; critérios de cor/export explícitos |
| **V2 · Post-V1 Candidate** | macOS/Windows, depois builds móveis/tablet conforme demanda | Platform ports sem importar GUI no domínio; critérios de IME/pen/accessibility por sistema |
| **V3 · Research** | Colaboração offline-first, sync, comentários, versioning e componentes compartilhados | Modelo de operações/IDs/recursos adequado; avaliação CRDT/OT, conflitos de geometria, autorização e privacidade |
| **V3 · Post-V1 Candidate** | Plugin marketplace, API/SDK estáveis, macros visuais, MCP remoto e extensões de formato | Quotas, revisão/protocolo estável, isolamento e matriz de permissões |
| **Research** | Vetorização assistida, segmentação, redução de nós por erro, recuperação de constraints e acessibilidade assistida | Resultado editável, medida de qualidade/erro, consentimento/licença/modelo local quando aplicável |
| **Research** | Recuperação determinística de sessões, replay/perf profiler embutido, presets de orçamento por máquina | Logs sem artwork/segredos, overhead medido e ferramenta útil ao usuário/desenvolvedor |

Nenhuma ideia futura bloqueia o MVP; nenhuma justifica tirar CMYK/PDF da V1. Introduzir somente depois de validar problema, custo, algoritmo, licença, comportamento live, UX e critérios de aceitação.

## Orçamentos de desempenho e memória propostos

São metas de engenharia a calibrar, não resultados já obtidos. Declarar CPU/GPU/driver/monitor/OS/build release, resolução, zoom e quantidade de objetos visíveis. Usar median/p95/p99, aquecimento e amostras suficientes; gravar RSS, VRAM, alocações e efeito de export concorrente.

| Cenário | Meta inicial | Limite/observação |
| --- | --- | --- |
| Interação/preview 60 Hz | Frame p95 ≤16,7 ms; tentar manter trabalho UI ≤8 ms | Medir input-to-paint separado; frame não é só canvas_snapshot |
| Seleção/mover em 10.000 shapes | Hit/snapping p95 ≤8 ms; edição regional sem full rebuild | Incluir 1%/100% visível, hierarquia e efeitos; perf antes/depois |
| Brush em imagem de 24 MP | Feedback visual p95 ≤25 ms; sem ler arquivo no frame | Pressão real, múltiplos tiles, máscaras, preview/export concorrentes |
| Cache/histórico | Valores iniciais configuráveis de 512 MiB cada; total sob orçamento do processo | Não é garantia de RSS; dados originais, snapshots e buffers intermediários contam separadamente; eviction pode reduzir qualidade de cache, nunca arte |
| Documento grande | 10.000 paths + texto; raster adicional 100 MP com viewport parcial | Sem OOM; diagnóstico para operações acima do orçamento; spill/streaming onde necessário |
| Arquivos/export | UI responsiva, progresso/cancelamento e memória previsível | Definir throughput por formato após renderer correto; saída 300 DPI não pode alocar pasteboard vazio |

Corpus mínimo: vazio/degenerado; compound/holes; grupos transformados; múltiplos paints/masks; transparência parcial; offscreen blur/shadow; texto complexo; perfil distinto; imagens 8/16-bit; origens negativas; arquivos inválidos e save sob falha. Benchmarks release medem o caminho completo e são separados de unit tests; não depender de um assert debug de 50 ms.

## Backlog inicial e gestão de entregas

Começar por PRs pequenos com regressão focal: **(1)** F02/F03; **(2)** F17/F18/F19; **(3)** F09/F10/F11; **(4)** F01/F22 com migração; **(5)** F23/F24/F37; então consolidar M1 e as ferramentas. Mudanças de contrato trazem ADR/Atlas/espelhos na mesma entrega, sem inventar um novo documento canônico concorrente.

Papéis necessários: domínio/geometria; rendering/raster/performance; cor/PDF/texto; GUI/UX/Linux; QA/fixtures/integração. São responsabilidades, não pedido de criar agentes. Uma pessoa pode acumular papéis, mas validação colorimétrica/impressão e UX precisam de usuários/equipamentos representativos.

Classificar tamanho após spikes com critérios concretos: S até poucos dias para correção localizada; M até duas semanas para fluxo conhecido; L várias semanas para subsistema; XL contratos/fidelidade/ICC/PixelLayer, decompondo em entregas antes de estimar prazo. Não há base nesta auditoria para prometer calendário ou somar essas classes em uma data de release.

Cada item registra: ID e achados, versão/status, proprietário, dependências, ADRs/arquivos, resultado visível, invariantes, algoritmo/referência consultada, fixture, testes, performance, migração, i18n/a11y, riscos e evidência. Reavaliar o escopo com projetos reais ao final de M1, MVP e V1-B; aceitar novas prioridades sem afrouxar os gates de integridade e fidelidade.
