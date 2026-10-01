# Editores de objetos — implementação UI/UX

Data: 2026-10-01. Escopo: **Milestone Required (MVP)**. Esta segunda etapa corrige erros concretos dos editores; não conclui o inspetor, a edição direta de texto ou o MVP.

Renomear camada agora abre um campo editável em vez de alternar um asterisco. Cancelar, Escape e o fundo do modal dispensam sem comando. O nome exige 1–256 caracteres após remover espaços nas extremidades, sem caracteres de controle. O botão pode receber foco pelo teclado; ações de camada interrompem a propagação do clique e preservam a seleção anterior.

O texto no inspetor e no canvas usa o mesmo diálogo multilinha. O rascunho pertence à identidade estável de documento/objeto, recebe conteúdo ao abrir e não escreve durante renderização. Texto vazio permanece vazio ao reabrir. Aplicar troca o conteúdo numa cópia do descritor inicial; família, tamanho, altura de linha, espaçamento e vínculo ao caminho continuam intactos. Os passos de tamanho mudam apenas o tamanho. Este é um diálogo de conteúdo; cursor direto no canvas e tipografia profissional continuam pendentes.

A transformação precisa abre campos X, Y, largura, altura e rotação. Eles editam o posicionamento local em relação ao pai/prancheta, sem afirmar coordenadas globais. Coordenadas e dimensões usam pontos; rotação usa graus, convertidos aos radianos do domínio. Vírgula decimal e sufixos `pt`/`deg`/`°` são aceitos. Unidades erradas, valores vazios/não finitos e dimensões não positivas são rejeitados antes do comando. O rascunho fica aberto com erro localizado por campo. Alterar apenas a largura preserva exatamente a rotação existente; foram removidos os antigos botões ±10 que zeravam a rotação. Transformação precisa e edição de texto exigem um objeto selecionado. Um aviso explica que os demais controles legados ainda afetam o primeiro objeto; a multisseleção permanece incompleta.

Cada diálogo compara a propriedade editada com o valor inicial antes de publicar. Objetos fechados, outra sessão ativa (mesmo com IDs reutilizados) ou edição conflitante falham sem sobrescrever o trabalho mais novo. Alterações independentes, como opacidade, ficam intactas. Aplicar envia um comando reversível pela ponte; valores inalterados não geram histórico. Falhas mantêm o editor e o rascunho. Modais bloqueiam atalhos de canvas pelo gate existente. Os passos de largura do traço também preservam o token de cor existente.

O texto visível usa o catálogo canônico EN e espelho PT. A [ADR-008](/pt/developers/adr/ADR-008-object-edit-drafts) define o rascunho e a publicação. A [evidência de fontes/checks](/implementation/uiux-object-edits.json) registra os checks executados e hashes exatos da cópia usada.

## Mapeamento do Atlas e da auditoria

| Requisito | Mudança e limite |
| --- | --- |
| 08.5 controles; 08.17 painéis | Nome, conteúdo multilinha, posicionamento numérico local; framework completo de propriedades pendente |
| 08.18 diálogos; 08.19 texto/unidades | Aplicar/Cancelar explícitos, erros preservados, unidades declaradas e EN/PT |
| 08.14 entrada; 08.16 UI gauntlet | Bloqueio de atalhos nos modais e interação headless; AT-SPI/IME reais pendentes |
| UX-08 / UX-10–13 | Renomear fictício, medidas, rotação, rascunho compartilhado e perda da tipografia corrigidos neste fluxo |
| UX-09 / UX-14 | Multisseleção e edição direta no canvas permanecem abertas |

## O que falta no MVP e na V1

A tabela registra aceite restante; não afirma que o código de backend desenvolvido em paralelo esteja ausente. O escopo completo continua no [plano MVP–V1](/pt/developers/implementation-roadmap-2026-09-30).

| Versão | Trabalho restante |
| --- | --- |
| MVP | Multisseleção/valores mistos, árvore de camadas com recolher/filtro, propriedades de formas/gradientes/traços e previews |
| MVP | Cursor/seleção de texto no canvas, edição coerente com shaping, bidi/grafemas, IME real e controles de fontes |
| MVP | Tema/sistema, densidade, contraste, alvos e consistência completa de EN/PT/tokens |
| MVP | Fluxos completos de vetores, bitmap persistente/máscaras, preview, undo/redo, reabertura e exportações fiéis |
| MVP | Aceite de recuperação, relatório de degradação, operações longas canceláveis e orçamentos de recursos |
| MVP | Wayland/X11, portais, leitor de tela/AT-SPI, foco, HiDPI, tablet, pacote Linux e gates de desempenho/release |
| V1 | ICC/CMYK verdadeiro, prova de cor, PDF/X-4 profissional e preflight de impressão |
| V1 | Tipografia/bitmap profissionais, aparência live avançada, símbolos/assets e interoperabilidade no escopo definido |

## Limites da verificação

Os checks usam uma cópia congelada das fontes e cache Cargo separado, sem interferir nos artefatos do backend. As regressões verificam erros, preservação de estilos/caminho, entrada vazia/multilinha real, propagação de seleção, revisão/undo/redo, colisões de IDs entre documentos e conflitos. Suíte completa do desktop, localização, Clippy estrito do aplicativo, formatação e documentação são registrados quando executados. As capturas headless são renders Freya reais, não os wireframes HTML anteriores.

Esses checks não certificam todo o MVP em evolução. Rótulos/foco do leitor de tela, Wayland/X11, pressão/inclinação, portais, composição IME e usabilidade por tarefas exigem aceite real. O editor de conteúdo acrescenta Aplicar deliberadamente; edição visual direta permanece no backlog. O CLI Prumo estava indisponível; objetivo, contratos, escopo, hashes e checks compõem o microcontexto local.
