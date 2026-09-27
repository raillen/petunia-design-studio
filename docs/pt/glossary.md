# Glossário (EN ↔ PT-BR)

Vocabulário canônico bilíngue. **O inglês é normativo**: em divergência, a definição em inglês vence. Status: `preferred` (preferido), `deprecated` (mantido, não usar em trabalho novo), `forbidden` (nunca emitir).

## Modelo central

| EN (canônico) | PT-BR | Status | Definição |
| ------------- | ----- | ------ | --------- |
| Surface | Superfície | preferred | Região unificada de prancheta/página/exportação conforme o contexto. |
| Artboard | Prancheta | preferred | Superfície de design para composição vetorial/layout; uma Surface em contexto de prancheta. |
| Layer | Camada | preferred | Papel de contêiner na árvore única de documento; papel do painel Camadas, não hierarquia paralela. |
| Mask | Máscara | preferred | Controle não destrutivo de recorte/alfa; pixels originais preservados. |
| Symbol | Símbolo | preferred | Definição reutilizável de layout/conteúdo; substituto V1 das Master Pages. |
| SurfaceTemplate | Modelo de superfície | preferred | Layout reutilizável e leve de superfície, construído sobre Símbolos. |
| Adjustment | Ajuste | preferred | Nó reordenável, mascarável e não destrutivo de correção raster/cor. |
| Swatch | Amostra | preferred | Definição nomeada e reutilizável de cor no sistema semântico. |
| Persona | Persona | preferred | Composição de ferramentas/painéis/ações de Design ou Photo sobre um documento compartilhado. |
| Profile | Perfil | preferred | Composição nomeada de workspace misto referenciando IDs semânticos. |
| Binding | Vínculo | preferred | Ligação entre uma propriedade do documento e um campo de dados variáveis. |
| Record | Registro | preferred | Uma linha da fonte de dados variáveis usada na geração do Data Merge. |

## Identidade e automação

| EN (canônico) | PT-BR | Status | Definição |
| ------------- | ----- | ------ | --------- |
| ObjectId | ObjectId | preferred | Identidade estável tipada de um objeto; nunca índice de Vec ou ponteiro. |
| SurfaceId | SurfaceId | preferred | Identidade estável tipada de uma superfície. |
| ActionId | ActionId | preferred | Identificador semântico estável de uma operação (`aubrieta.*` lido, `ptnd.*` emitido). |
| TextId | TextId | preferred | Identificador semântico de texto de UI localizável; sem strings literais nas features. |
| IconId | IconId | preferred | Identificador de ícone independente de toolkit, com namespace. |
| Action | Ação | preferred | Intenção do usuário despachada pela sessão. |
| Command | Comando | preferred | Unidade validada de mutação aplicada via DocumentMutator. |
| ChangeSet | ChangeSet | preferred | Resultado atômico de um Comando: a única coisa que toca o armazenamento. |
| DocumentMutator | DocumentMutator | preferred | Escritor único do armazenamento; toda mutação passa por ele. |
| EffectChain | Cadeia de efeitos | preferred | Lista ordenada tipada de modificadores vivos avaliados sobre a geometria base. |

## Produto e formato

| EN (canônico) | PT-BR | Status | Definição |
| ------------- | ----- | ------ | --------- |
| Petunia Design Studio | Petunia Design Studio | preferred | Nome canônico da aplicação (app desktop Design + Photo). Nomes de produto não se traduzem. |
| .ptnd | .ptnd | preferred | Extensão nativa canônica; formato de pacote ZIP aberto. |
| .aubrieta / .aubri | .aubrieta / .aubri | deprecated | Apelidos legados; ainda abrem, nunca emitidos em salvamentos novos. |
| Master Pages | Páginas-mestre | deprecated | Fora do escopo V1; usar Símbolos + Modelo de superfície. |
| .abrt | .abrt | forbidden | Rejeitado: colide com o ecossistema ABRT do Fedora/RHEL. |
| .pds | .pds | forbidden | Candidato rejeitado de extensão nativa. |

## Processo

| EN (canônico) | PT-BR | Status | Definição |
| ------------- | ----- | ------ | --------- |
| Gauntlet | Gauntlet | preferred | A suíte de verificação imposta (`cargo xtask gauntlet`); "gauntlet" não se traduz. |
| Capability | Capacidade | preferred | Entrada de registro declarando o que uma feature provê; ausente = desabilitado com motivo. |
| Preflight | Pré-voo | preferred | Verificações de exportação (fontes, gama, orçamento raster) antes de materializar bytes. |
| Bake | Consolidar (bake) | preferred | Congelar explicitamente modificadores vivos na geometria base; sempre operação do usuário. O verbo *bake* é mantido sem tradução nos rótulos de UI. |

> Fonte de verdade dos termos de produto: `docs/glossary.json` no repositório. Esta página renderiza esse registro para humanos; em conflito, o JSON vence para status `preferred/forbidden`.
