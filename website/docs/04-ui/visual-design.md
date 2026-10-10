# Design visual, personas e iconografia

**Estado: GUI funcional implementada e verificada em Linux/offscreen — 2026-10-10.** Direção baseada nas duas imagens de referência Affinity fornecidas pelo mantenedor e na escolha de personas customizáveis, visual limpo, feedback visual e famílias Phosphor/Tabler selecionáveis. A direção foi implementada em `petunia-desktop`; execução e limites estão em [GUI nativa](#/docs/04-ui/native-desktop.md). O tracker distingue a entrega funcional de QA multiplataforma e tecnologias assistivas.

Este contrato complementa [Workspace](#/docs/04-ui/workspace.md) e o [ADR-0012](#/docs/00-architecture/adr/0012-interaction-contract.md), sem alterar suas decisões D1–D10. O [manifesto](#/docs/00-philosophy/manifesto.md) e a [acessibilidade](#/docs/04-ui/accessibility.md) governam a adaptação.

## Direção visual

As referências mostram um seletor superior de personas com ícone e nome, uma lista de visibilidade com switches e a mesma composição de editor em superfícies claras e escuras. A adaptação Petunia mantém o canvas como foco, organização previsível e identificação fácil da atividade atual.

- Superfícies neutras, separadas por bordas e pequenas diferenças de luminância; sombras discretas somente em superfícies sobrepostas.
- Barra de ferramentas à esquerda, canvas ao centro e painéis contextuais à direita, conforme ADR-0012 D9.
- Persona ativa indicada por ícone, nome e estado selecionado. Cores de identidade são opcionais e localizadas no badge da persona; o único accent de interação permanece compartilhado, conforme D6.
- No máximo três ações dominantes na barra contextual; controles avançados em expansão explícita. Hover não reorganiza o layout.
- Previews, seleção e operações ativas aparecem perto do alvo. Notificações transitórias ficam reservadas para eventos que precisam chamar atenção fora desse contexto.

Não reproduzir a marca, o logotipo ou os assets Affinity. As imagens são referências de composição, não fontes de ícones redistribuíveis. Os grandes tamanhos da primeira imagem não são medidas da GUI: as medidas abaixo orientam os tokens executáveis da GUI nativa.

## Personas: atividade, visibilidade e personalização

**Persona** é um perfil de ferramentas e arranjo de painéis para uma atividade. Não é um documento separado nem um limite para acessar recursos: a paleta de comandos conserva as ações disponíveis da aplicação.

Perfis iniciais de direção: **Vetor, Pixel e Layout**, com nome localizado e identidade estável. Outros perfis e ferramentas dependem de capacidades reais do produto; a referência não autoriza apresentar ferramentas indisponíveis como funcionais.

| Superfície | Ação | Estado comunicado |
|---|---|---|
| Seletor superior | Ativar uma persona | Qual atividade está em uso; uma única persona ativa |
| Gerenciar personas | Mostrar ou ocultar uma persona | Se aparece no seletor; switch com nome acessível `Mostrar persona Vetor` |
| Editar persona | Renomear, escolher ícone e organizar ferramentas/painéis | Preview do arranjo antes de aplicar |
| Duplicar persona | Criar perfil personalizado a partir de um existente | Novo ID estável e nome editável |
| Restaurar perfil | Recuperar o arranjo padrão do perfil | Confirmação com escopo explícito; não restaura todo o aplicativo |

Regras de comportamento:

1. Ícone e nome permanecem visíveis no seletor. Quando faltar espaço, personas excedentes vão para um menu nomeado; a persona ativa continua identificável.
2. Switches de visibilidade não ativam personas. Ocultar a persona ativa exige primeiro ativar outra; exibir o motivo no controle. A última persona visível não pode ser ocultada.
3. Ativação restaura ferramentas e painéis do perfil, preservando o documento, histórico, zoom/pan e seleção quando válida. Uma interação ou draft pendente segue o contrato de Commit/Cancel vigente antes de mudar contexto; nunca há confirmação ou descarte silencioso.
4. Personalização tem `Aplicar`, `Cancelar` e retorno de foco ao acionador. Preview não altera o documento. Exclusão de perfil personalizado usa confirmação; excluir o perfil ativo exige escolher outro primeiro.
5. Perfis customizados podem ser reordenados por controles de teclado, além de arraste. Não depender de drag para editar o seletor.
6. Aparência, família de ícones e estilo são preferências independentes da persona. Trocar de persona não altera tema, iconografia ou atalhos globais silenciosamente.

Layout e preferências vivem na UI/workspace, nunca no PTND. Separar o arranjo salvo do perfil do estado transitório de interação; restaurar um perfil não restaura um drag pendente.

## Customizações independentes

| Preferência | Opções | Padrão inicial proposto |
|---|---|---|
| Aparência | Claro, escuro, alto contraste | Escuro; restaurar escolha explícita nas próximas sessões |
| Família de ícones | Phosphor, Tabler | Phosphor |
| Estilo | Outline (contorno), fill (preenchido) | Outline |
| Densidade | Confortável, compacta | Confortável |
| Identidade da persona | Nome, ícone, badge opcional | Nome e ícone; badge discreto |
| Movimento | Seguir preferência de redução do sistema; permitir reduzir manualmente | Sem movimento essencial à compreensão |

Preferências de acessibilidade prevalecem sobre densidade e ornamentação. Alto contraste pode substituir cores de badges para assegurar legibilidade; nome, ícone e estado permanecem. Os controles são independentes: escolher Tabler não força aparência ou densidade.

## Iconografia e resolução de assets

As famílias aprovadas são [Phosphor](https://github.com/phosphor-icons/core) e [Tabler Icons](https://github.com/tabler/tabler-icons). Ambas possuem assets SVG e licença MIT; selecionar somente o conjunto efetivamente necessário e manter licença/copyright junto aos recursos distribuídos.

Na UI, cada ação usa uma chave semântica estável, por exemplo `tool.select` ou `document.export`. Um catálogo resolve essa chave para o SVG da família/estilo escolhidos. Nome, shortcut, disponibilidade e execução continuam pertencendo ao registry de ações descrito em Workspace, independentemente do desenho.

| Estilo da preferência | Phosphor | Tabler |
|---|---|---|
| Outline | `regular` | `outline` |
| Fill | `fill` | `filled`, quando disponível |

O catálogo filled do Tabler é menor que o outline. Quando não houver variante preenchida, usar o outline equivalente **da mesma família**; a prévia de customização informa que alguns símbolos permanecem em contorno. Se faltar o símbolo nas duas variantes, usar um asset Petunia previamente revisado e registrado para aquela chave, sem escolher outro ícone por aproximação de nome.

- Não gerar fill trocando mecanicamente `fill="none"`: linhas de um outline não definem necessariamente áreas válidas de preenchimento.
- Não misturar famílias por estado, painel ou fallback. Ícones Petunia exclusivos devem seguir a geometria de cada família e manter a mesma semântica.
- Estilo fill não significa ação ativa: seleção é indicada também pelo container, marcador e semântica acessível, inclusive quando todos os ícones estão preenchidos.
- Normalizar viewBox/área óptica e margens no catálogo. Não prometer traços idênticos entre Phosphor e Tabler sem ajuste visual.
- SVGs são recursos locais Qt/QML, com versão upstream fixada e lista de origem por asset. Sem CDN, webfont, React ou acesso de rede em runtime.
- Cores dos ícones acompanham tokens de foreground/state. A estratégia de tint e a compatibilidade SVG devem ser verificadas no renderer Qt escolhido, sem assumir suporte a CSS web ou `currentColor`.
- Assets de persona usam chave própria, além de nome acessível; trocar a família preserva o significado e a posição das ferramentas.

Os assets importados estão registrados no catálogo versionado de `petunia-desktop`, conforme [política de dependências](#/docs/00-philosophy/dependency-policy.md); origem e checksums são verificados por `scripts/verify-icons.py`.

## Tokens e escala inicial

**Tokens** são valores nomeados que separam aparência de comportamento. A fonte deve ser um catálogo JSON na camada UI, com valores base, papéis semânticos e tokens de componente; QML consome a projeção desse catálogo. Cores de interface não entram no modelo de cor artístico.

Paleta inicial proposta para superfícies opacas, ainda sujeita à calibração no protótipo:

| Papel | Claro | Escuro | Alto contraste |
|---|---|---|---|
| `surface.canvasSurround` | `#E9EDF1` | `#161A1F` | `#000000` |
| `surface.panel` | `#F7F8FA` | `#20252B` | `#000000` |
| `surface.raised` | `#FFFFFF` | `#2A3038` | `#000000` |
| `foreground.primary` | `#18212B` | `#F3F5F7` | `#FFFFFF` |
| `foreground.secondary` | `#57606A` | `#B2BBC5` | `#FFFFFF` |
| `border.subtle` | `#CCD3DA` | `#414B57` | `#FFFFFF` |
| `border.control` | `#6B7580` | `#83909E` | `#FFFFFF` |
| `accent.interaction` | `#005EA8` | `#66D9EF` | `#FFFF00` |
| `state.selectedBackground` | `#DFEDFA` | `#153D49` | `#000000` |
| `state.selectedForeground` | `#06447A` | `#D8F7FC` | `#FFFFFF` |
| `focus.ring` | `#005EA8` | `#66D9EF` | `#FFFF00` |

`border.subtle` organiza superfícies; não é o único indicador de um controle ou estado. `border.control`, foreground e foco atendem contraste não textual nos fundos previstos. Seleção em alto contraste usa contorno, marcador e semântica; foco usa anel externo distinto. Cor de erro/sucesso exige token próprio, símbolo e texto, com contraste medido antes da integração.

| Medida | Base proposta, em px lógicos |
|---|---|
| Grade de espaçamento | 4; intervalos usuais 8, 12 e 16 |
| Texto de controles / texto secundário | 14 / 13; fonte de UI do sistema, escalável |
| Ícones de ferramentas / persona | 20 / 20–24 |
| Botão de ferramenta confortável / compacto | 40×40 / 32×32; área de captura separada do desenho |
| Barra superior / contextual | 48–56 / 40–48; expandir com escala de texto |
| Raio de botão / painel sobreposto | 6 / 8 |
| Anel de foco | 2, com separação visual do estado selecionado |

São medidas iniciais, não alturas rígidas: texto ampliado não pode ser cortado. Ícone visual de 16 é reservado a controles compactos após validação. Os targets/markers geométricos do canvas continuam sob ADR-0012 D3; não aplicar medidas de botão aos nodes.

## Feedback visual e acessível

| Estado | Apresentação e comportamento |
|---|---|
| Normal | Contraste legível, nome ou tooltip; sem competir com o canvas |
| Hover | Fundo/borda discretos; sem mudar ferramenta, seleção, layout ou documento |
| Foco | Anel visível por teclado, independente de hover/seleção |
| Ativo/selecionado | Container destacado, marcador e estado acessível; não depender de cor ou fill |
| Preview | Resultado provisório identificável, com caminho de Confirmar/Cancelar conforme a ferramenta |
| Indisponível | Motivo consultável pelo foco, ajuda contextual ou descrição; não só ícone apagado |
| Processando | Progresso real quando disponível; indicador indeterminado caso contrário; cancelamento para jobs canceláveis |
| Confirmado/cancelado | Retorno contextual breve, preservando orientação e foco |
| Erro | O que ocorreu, impacto e recuperação; diagnóstico técnico em expansão |

Não emitir toast a cada movimento, snap ou troca de seleção. Durante drag, comunicar mudanças significativas e início/fim da operação; evitar inundar leitores de tela. Respeitar preferência de movimento reduzido sem perder informação.

Personas e customizações seguem navegação de teclado previsível e semântica Qt nativa: nome, papel, selecionado/marcado, disponibilidade e valor. O popover devolve foco ao acionador ao fechar; Escape primeiro cancela a edição local quando houver. Tooltips seguem D5: nome + atalho atual, 400 ms em hover e acesso imediato por foco. A paleta de comandos conserva descoberta por teclado.

## Gates e evidência

A entrega nativa mantém estes gates separados, sem atribuir conformidade completa por capturas.

| Gate | Evidência de execução em 2026-10-10 |
|---|---|
| Workspace Rust headless | 537 testes passaram, zero falhas; Clippy de todos os targets com warnings negados passou |
| QML: teclado, precisão, cancelamento, foco e docking | 26 resultados QtQuickTest PASS, zero falhas; lint das 11 fontes QML sem avisos |
| SVGs locais | 154 SVGs embarcados; 18 fallbacks fill → outline da mesma família; 172 entradas com origem/SHA-256, avisos MIT preservados |
| Preferências e personas | Roundtrip, recuperação, validação, gravação atômica e escala 1–2 cobertos em testes Rust |
| Janela/backend/arquivos | Build Qt 6.8.2/CXX-Qt e salvamento/reabertura/exportação passaram em DPI 1 e 2 nos três temas; comparação de 81 pixels do canvas confirma a apresentação capturada |
| Tecnologias assistivas e plataformas | AT-SPI/NVDA/VoiceOver, GPU e dispositivos de caneta ainda exigem QA específico |

Tokens executáveis: `crates/petunia-desktop/qml/design-tokens.json`; o componente `StudioTheme.qml` é gerado desse catálogo. As paletas são opacas, com foreground, borda e foco explícitos. A revisão calculou mínimos de texto/controles de 5,43/3,98:1 no claro, 6,85/4,09:1 no escuro e 12,37/19,56:1 em alto contraste. Os valores não certificam integração com leitores de tela.

O aplicativo não usa animação de conteúdo. A preferência manual de redução de movimento é persistida; descoberta automática da preferência do sistema continua pendente. O estado flutuante do painel permanece na sessão, enquanto ferramentas/painéis habilitados do perfil são salvos.

Critérios de aceitação para QA nas plataformas finais:

1. As quatro combinações Phosphor/Tabler × outline/fill preservam nomes, comandos e estados, incluindo fallback de variantes ausentes.
2. Claro, escuro e alto contraste apresentam texto normal com contraste mínimo 4,5:1 e indicadores essenciais com 3:1 nos fundos reais. Não declarar conformidade por screenshot ou paleta isolada.
3. Criar, duplicar, renomear, reordenar, ocultar e restaurar personas funciona por teclado; a ativa permanece identificável, sem perder documento ou edição pendente.
4. Densidades e texto ampliado funcionam na janela-base 1024×640, com reflow abaixo disso; confirmar DPI 1 e 2 e escala de texto até 200%.
5. Foco, seleção e hover são distintos nos três temas; leitores de tela recebem atividade, visibilidade e mudanças significativas sem spam.
6. Preferências de UI são versionadas fora do PTND; recurso ou perfil desconhecido usa padrão válido, comunica recuperação e preserva documento.

**Tracker:** U01 continua IN PROGRESS. A GUI funcional tem evidência própria; QA nativo com tecnologias assistivas e plataformas não é fechado pelos testes offscreen.
