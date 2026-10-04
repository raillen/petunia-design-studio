# Studio — análise e implementação de UI/UX

Data: 2026-10-04. Escopo: **Milestone Required (MVP)**, base compartilhada com V1. [ADR-012](/pt/developers/adr/ADR-012-studio-workspace) define o contrato; [evidência](/implementation/uiux-studio.json) registra execução real.

A reorganização usa Affinity como referência de estrutura e hierarquia, preservando os controles e identidade Petunia. A análise e as correções cobrem o shell, ferramentas, documentos, painéis, cor, navegação, formulários, preferências e retorno de operações.

| Problema observado | Comportamento implementado |
| --- | --- |
| Mistura de comandos, aparência e contexto | Menus, Studio, documentos, contexto e status têm regiões próprias |
| Cor e camadas concorriam pela mesma aba | Cor/Amostras ficam independentes da inspeção inferior |
| Ferramentas em duas colunas viravam uma linha | Grade real com rolagem e ferramentas de pintura Pixel |
| Painéis excediam o canvas e reabrir divisores falhava | Largura/altura limitadas, hooks estáveis, captura do arraste e teclado |
| Callbacks mantinham a primeira prévia | Novos quadros, câmera e sobreposições publicados por época de apresentação |
| Abas/controles longos excediam a janela | Rolagem, elipse, larguras limitadas e recolhimento |
| Camadas sem recolhimento ou nomes claros | Árvore por identidade de sessão/objeto, ícones semânticos e ações independentes |
| Amostras apagavam a espessura e ignoravam raster | Multisseleção vetorial atômica; cor do pincel raster; alpha/espessura preservados |
| Favoritos adicionavam rosa fixo | Adiciona e deduplica a cor atual; mantém favoritos na sessão do app |
| Navegador mostrava caixas em vez da composição | Worker real e área da câmera; navegação por clique, arraste e setas |
| Documento novo permitia apenas incrementos | Dimensões digitáveis, presets, orientação e erros antes de criar |
| Preferências e tarefas anunciavam estados fictícios | Descrição do renderizador real e jobs reais |
| Ícones preenchidos mudavam de significado | Contorno equivalente quando não há preenchido semântico |
| Ícones sem identidade e atalhos roubados por campos | Nome/estado/foco acessíveis e isolamento do teclado |
| Cópia fixa em português | Catálogo EN/pt-BR responsivo ao idioma |

## Validação final

As verificações foram executadas após implementar o incremento; falhas encontradas na validação final foram corrigidas antes das execuções aprovadas registradas. `cargo xtask gauntlet` passou 951 testes do workspace em 104 alvos, sem falhas ou testes ignorados, Clippy estrito, formatação, arquitetura, conformidade CLI, quatro projetos do MVP e build da documentação. `cargo xtask ui-gauntlet` passou os 84 testes desktop, já incluídos no total. A documentação contém 35 páginas canônicas e 35 espelhos pt-BR. A inspeção independente pypdf/Poppler preservou tinta CMYK, alpha e ICC originais nos quatro PDFs de teste.

As capturas são renderizações da interface Rust/Freya de produção, com conteúdo original de teste, sem reconstrução HTML. Dezoito capturas Studio cobrem os dois idiomas em quatro tamanhos e interações específicas; nove capturas adicionais exercitam módulos desktop existentes. O registro informa comandos, hashes de fontes/artefatos e limites. Os números da baseline ADR-011 são evidência histórica.

As capturas binárias e os PDFs/TIFFs de teste são artefatos da execução local. Seus hashes e dimensões estão no registro; o repositório contém fontes da implementação e relatórios textuais da validação. Reproduza os artefatos após instalar as dependências nativas documentadas:

```sh
PETUNIA_UI_EVIDENCE_DIR="$PWD/docs/public/implementation/studio/artifacts" \
PETUNIA_CMYK_EVIDENCE_DIR="$PWD/docs/public/implementation/studio/cmyk-artifacts" \
cargo xtask gauntlet
python3 scripts/verify-native-cmyk-pdf.py docs/public/implementation/studio/cmyk-artifacts
```

## Limites de aceitação

Enquadramento automático ocorre uma vez por sessão. Favoritos são da sessão do app. Transformação numérica exata continua exigindo uma seleção. Várias propriedades legadas ainda editam apenas o primeiro objeto. O gerenciador de docking livre, persistência das preferências, tema claro/sistema, estilos spot certificados e Studio Layout não são implementados neste contrato. Os requisitos profissionais de impressão/tipografia e a aceitação externa de hardware, acessibilidade e usabilidade permanecem no [roadmap](/pt/developers/implementation-roadmap-2026-09-30).
